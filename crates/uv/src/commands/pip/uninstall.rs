use std::fmt::Write;

use anyhow::Result;
use itertools::{Either, Itertools};
use owo_colors::OwoColorize;
use tracing::{debug, warn};

use uv_cache::Cache;
use uv_client::BaseClientBuilder;
use uv_configuration::{DryRun, KeyringProviderType, Preview};
use uv_distribution_types::Requirement;
use uv_distribution_types::{InstalledMetadata, Name, UnresolvedRequirement};
use uv_fs::Simplified;
use uv_pep508::UnnamedRequirement;
use uv_pypi_types::VerbatimParsedUrl;
use uv_python::EnvironmentPreference;
use uv_python::PythonRequest;
use uv_python::{Prefix, PythonEnvironment, Target};
use uv_requirements::{RequirementsSource, RequirementsSpecification};

use crate::commands::pip::operations::report_target_environment;
use crate::commands::{ExitStatus, elapsed};
use crate::printer::Printer;
use crate::settings::NetworkSettings;

/// Uninstall packages from the current environment.
#[allow(clippy::fn_params_excessive_bools)]
pub(crate) async fn pip_uninstall(
    sources: &[RequirementsSource],
    python: Option<String>,
    system: bool,
    break_system_packages: bool,
    target: Option<Target>,
    prefix: Option<Prefix>,
    cache: Cache,
    keyring_provider: KeyringProviderType,
    network_settings: &NetworkSettings,
    dry_run: DryRun,
    printer: Printer,
    preview: Preview,
) -> Result<ExitStatus> {
    let start = std::time::Instant::now();

    let client_builder = BaseClientBuilder::new()
        .retries_from_env()?
        .connectivity(network_settings.connectivity)
        .native_tls(network_settings.native_tls)
        .keyring(keyring_provider)
        .allow_insecure_host(network_settings.allow_insecure_host.clone());

    // Read all requirements from the provided sources.
    let spec = RequirementsSpecification::from_simple_sources(sources, &client_builder).await?;

    // Detect the current Python interpreter.
    let environment = PythonEnvironment::find(
        &python
            .as_deref()
            .map(PythonRequest::parse)
            .unwrap_or_default(),
        EnvironmentPreference::from_system_flag(system, true),
        &cache,
        preview,
    )?;

    report_target_environment(&environment, &cache, printer)?;

    // Apply any `--target` or `--prefix` directories.
    let environment = if let Some(target) = target {
        debug!(
            "Using `--target` directory at {}",
            target.root().user_display()
        );
        environment.with_target(target)?
    } else if let Some(prefix) = prefix {
        debug!(
            "Using `--prefix` directory at {}",
            prefix.root().user_display()
        );
        environment.with_prefix(prefix)?
    } else {
        environment
    };

    // If the environment is externally managed, abort.
    if let Some(externally_managed) = environment.interpreter().is_externally_managed() {
        if break_system_packages {
            debug!("Ignoring externally managed environment due to `--break-system-packages`");
        } else {
            return if let Some(error) = externally_managed.into_error() {
                Err(anyhow::anyhow!(
                    "The interpreter at {} is externally managed, and indicates the following:\n\n{}\n\nConsider creating a virtual environment with `uv venv`.",
                    environment.root().user_display().cyan(),
                    textwrap::indent(&error, "  ").green(),
                ))
            } else {
                Err(anyhow::anyhow!(
                    "The interpreter at {} is externally managed. Instead, create a virtual environment with `uv venv`.",
                    environment.root().user_display().cyan()
                ))
            };
        }
    }

    let _lock = environment
        .lock()
        .await
        .inspect_err(|err| {
            warn!("Failed to acquire environment lock: {err}");
        })
        .ok();

    // Index the current `site-packages` directory.
    let site_packages = uv_installer::SitePackages::from_environment(&environment)?;

    // Partition the requirements into named and unnamed requirements.
    let (named, unnamed): (Vec<Requirement>, Vec<UnnamedRequirement<VerbatimParsedUrl>>) = spec
        .requirements
        .into_iter()
        .partition_map(|entry| match entry.requirement {
            UnresolvedRequirement::Named(requirement) => Either::Left(requirement),
            UnresolvedRequirement::Unnamed(requirement) => Either::Right(requirement),
        });

    // Sort and deduplicate the packages, which are keyed by name. Like `pip`, we ignore the
    // dependency specifier (even if it's a URL).
    let names = {
        let mut packages = named
            .into_iter()
            .map(|requirement| requirement.name)
            .collect::<Vec<_>>();
        packages.sort_unstable();
        packages.dedup();
        packages
    };

    // Sort and deduplicate the unnamed requirements, which are keyed by URL rather than package name.
    let urls = {
        let mut urls = unnamed
            .into_iter()
            .map(|requirement| requirement.url.verbatim.to_url())
            .collect::<Vec<_>>();
        urls.sort_unstable();
        urls.dedup();
        urls
    };

    // Map to the local distributions.
    let distributions = {
        let mut distributions = Vec::with_capacity(names.len() + urls.len());

        // Identify all packages that are installed.
        for package in &names {
            let installed = site_packages.get_packages(package);
            if installed.is_empty() {
                if !dry_run.enabled() {
                    writeln!(
                        printer.stderr(),
                        "{}{} Skipping {} as it is not installed",
                        "warning".yellow().bold(),
                        ":".bold(),
                        package.as_ref().bold()
                    )?;
                }
            } else {
                distributions.extend(installed);
            }
        }

        // Identify all unnamed distributions that are installed.
        for url in &urls {
            let installed = site_packages.get_urls(url);
            if installed.is_empty() {
                if !dry_run.enabled() {
                    writeln!(
                        printer.stderr(),
                        "{}{} Skipping {} as it is not installed",
                        "warning".yellow().bold(),
                        ":".bold(),
                        url.as_ref().bold()
                    )?;
                }
            } else {
                distributions.extend(installed);
            }
        }

        // Deduplicate, since a package could be listed both by name and editable URL.
        distributions.sort_unstable_by_key(|dist| dist.install_path());
        distributions.dedup_by_key(|dist| dist.install_path());
        distributions
    };

    if distributions.is_empty() {
        if dry_run.enabled() {
            writeln!(printer.stderr(), "Would make no changes")?;
        } else {
            writeln!(
                printer.stderr(),
                "{}{} No packages to uninstall",
                "warning".yellow().bold(),
                ":".bold(),
            )?;
        }
        return Ok(ExitStatus::Success);
    }

    // Uninstall each package.
    if !dry_run.enabled() {
        for distribution in &distributions {
            let summary = uv_installer::uninstall(distribution).await?;
            debug!(
                "Uninstalled {} ({} file{}, {} director{})",
                distribution.name(),
                summary.file_count,
                if summary.file_count == 1 { "" } else { "s" },
                summary.dir_count,
                if summary.dir_count == 1 { "y" } else { "ies" },
            );
        }
    }

    let uninstalls = distributions.len();
    let s = if uninstalls == 1 { "" } else { "s" };
    if dry_run.enabled() {
        writeln!(
            printer.stderr(),
            "{}",
            format!(
                "Would uninstall {}",
                format!("{uninstalls} package{s}").bold(),
            )
            .dimmed()
        )?;
    } else {
        writeln!(
            printer.stderr(),
            "{}",
            format!(
                "Uninstalled {} {}",
                format!("{uninstalls} package{s}").bold(),
                format!("in {}", elapsed(start.elapsed())).dimmed(),
            )
            .dimmed()
        )?;
    }

    for distribution in distributions {
        writeln!(
            printer.stderr(),
            " {} {}{}",
            "-".red(),
            distribution.name().as_ref().bold(),
            distribution.installed_version().to_string().dimmed()
        )?;
    }

    Ok(ExitStatus::Success)
}
