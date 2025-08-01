use std::collections::BTreeSet;
use std::fmt::Write;

use anyhow::Context;
use itertools::Itertools;
use owo_colors::OwoColorize;
use tracing::{Level, debug, enabled, warn};

use uv_cache::Cache;
use uv_client::{BaseClientBuilder, FlatIndexClient, RegistryClientBuilder};
use uv_configuration::{
    BuildOptions, Concurrency, ConfigSettings, Constraints, DryRun, ExtrasSpecification,
    HashCheckingMode, IndexStrategy, PackageConfigSettings, Preview, PreviewFeatures, Reinstall,
    SourceStrategy, Upgrade,
};
use uv_configuration::{KeyringProviderType, TargetTriple};
use uv_dispatch::{BuildDispatch, SharedState};
use uv_distribution_types::{
    DependencyMetadata, Index, IndexLocations, NameRequirementSpecification, Origin, Requirement,
    Resolution, UnresolvedRequirementSpecification,
};
use uv_fs::Simplified;
use uv_install_wheel::LinkMode;
use uv_installer::{SatisfiesResult, SitePackages};
use uv_normalize::{DefaultExtras, DefaultGroups};
use uv_pep508::PackageName;
use uv_pypi_types::Conflicts;
use uv_python::{
    EnvironmentPreference, Prefix, PythonEnvironment, PythonInstallation, PythonPreference,
    PythonRequest, PythonVersion, Target,
};
use uv_requirements::{GroupsSpecification, RequirementsSource, RequirementsSpecification};
use uv_resolver::{
    DependencyMode, ExcludeNewer, FlatIndex, OptionsBuilder, PrereleaseMode, PylockToml,
    PythonRequirement, ResolutionMode, ResolverEnvironment,
};
use uv_torch::{TorchMode, TorchStrategy};
use uv_types::{BuildIsolation, HashStrategy};
use uv_warnings::{warn_user, warn_user_once};
use uv_workspace::WorkspaceCache;
use uv_workspace::pyproject::ExtraBuildDependencies;

use crate::commands::pip::loggers::{DefaultInstallLogger, DefaultResolveLogger, InstallLogger};
use crate::commands::pip::operations::Modifications;
use crate::commands::pip::operations::{report_interpreter, report_target_environment};
use crate::commands::pip::{operations, resolution_markers, resolution_tags};
use crate::commands::{ExitStatus, diagnostics};
use crate::printer::Printer;
use crate::settings::NetworkSettings;

/// Install packages into the current environment.
#[allow(clippy::fn_params_excessive_bools)]
pub(crate) async fn pip_install(
    requirements: &[RequirementsSource],
    constraints: &[RequirementsSource],
    overrides: &[RequirementsSource],
    build_constraints: &[RequirementsSource],
    constraints_from_workspace: Vec<Requirement>,
    overrides_from_workspace: Vec<Requirement>,
    build_constraints_from_workspace: Vec<Requirement>,
    extras: &ExtrasSpecification,
    groups: &GroupsSpecification,
    resolution_mode: ResolutionMode,
    prerelease_mode: PrereleaseMode,
    dependency_mode: DependencyMode,
    upgrade: Upgrade,
    index_locations: IndexLocations,
    index_strategy: IndexStrategy,
    torch_backend: Option<TorchMode>,
    dependency_metadata: DependencyMetadata,
    keyring_provider: KeyringProviderType,
    network_settings: &NetworkSettings,
    reinstall: Reinstall,
    link_mode: LinkMode,
    compile: bool,
    hash_checking: Option<HashCheckingMode>,
    installer_metadata: bool,
    config_settings: &ConfigSettings,
    config_settings_package: &PackageConfigSettings,
    no_build_isolation: bool,
    no_build_isolation_package: Vec<PackageName>,
    extra_build_dependencies: &ExtraBuildDependencies,
    build_options: BuildOptions,
    modifications: Modifications,
    python_version: Option<PythonVersion>,
    python_platform: Option<TargetTriple>,
    strict: bool,
    exclude_newer: ExcludeNewer,
    sources: SourceStrategy,
    python: Option<String>,
    system: bool,
    break_system_packages: bool,
    target: Option<Target>,
    prefix: Option<Prefix>,
    python_preference: PythonPreference,
    concurrency: Concurrency,
    cache: Cache,
    dry_run: DryRun,
    printer: Printer,
    preview: Preview,
) -> anyhow::Result<ExitStatus> {
    let start = std::time::Instant::now();

    if !preview.is_enabled(PreviewFeatures::EXTRA_BUILD_DEPENDENCIES)
        && !extra_build_dependencies.is_empty()
    {
        warn_user_once!(
            "The `extra-build-dependencies` option is experimental and may change without warning. Pass `--preview-features {}` to disable this warning.",
            PreviewFeatures::EXTRA_BUILD_DEPENDENCIES
        );
    }

    let client_builder = BaseClientBuilder::new()
        .retries_from_env()?
        .connectivity(network_settings.connectivity)
        .native_tls(network_settings.native_tls)
        .keyring(keyring_provider)
        .allow_insecure_host(network_settings.allow_insecure_host.clone());

    // Read all requirements from the provided sources.
    let RequirementsSpecification {
        project,
        requirements,
        constraints,
        overrides,
        pylock,
        source_trees,
        groups,
        index_url,
        extra_index_urls,
        no_index,
        find_links,
        no_binary,
        no_build,
        extras: _,
    } = operations::read_requirements(
        requirements,
        constraints,
        overrides,
        extras,
        Some(groups),
        &client_builder,
    )
    .await?;

    if pylock.is_some() {
        if !preview.is_enabled(PreviewFeatures::PYLOCK) {
            warn_user!(
                "The `--pylock` setting is experimental and may change without warning. Pass `--preview-features {}` to disable this warning.",
                PreviewFeatures::PYLOCK
            );
        }
    }

    let constraints: Vec<NameRequirementSpecification> = constraints
        .iter()
        .cloned()
        .chain(
            constraints_from_workspace
                .into_iter()
                .map(NameRequirementSpecification::from),
        )
        .collect();

    let overrides: Vec<UnresolvedRequirementSpecification> = overrides
        .iter()
        .cloned()
        .chain(
            overrides_from_workspace
                .into_iter()
                .map(UnresolvedRequirementSpecification::from),
        )
        .collect();

    // Read build constraints.
    let build_constraints: Vec<NameRequirementSpecification> =
        operations::read_constraints(build_constraints, &client_builder)
            .await?
            .into_iter()
            .chain(
                build_constraints_from_workspace
                    .iter()
                    .cloned()
                    .map(NameRequirementSpecification::from),
            )
            .collect();

    // Detect the current Python interpreter.
    let environment = if target.is_some() || prefix.is_some() {
        let installation = PythonInstallation::find(
            &python
                .as_deref()
                .map(PythonRequest::parse)
                .unwrap_or_default(),
            EnvironmentPreference::from_system_flag(system, false),
            python_preference,
            &cache,
            preview,
        )?;
        report_interpreter(&installation, true, printer)?;
        PythonEnvironment::from_installation(installation)
    } else {
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
        environment
    };

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

    // Determine the markers to use for the resolution.
    let interpreter = environment.interpreter();
    let marker_env = resolution_markers(
        python_version.as_ref(),
        python_platform.as_ref(),
        interpreter,
    );

    // Determine the set of installed packages.
    let site_packages = SitePackages::from_environment(&environment)?;

    // Check if the current environment satisfies the requirements.
    // Ideally, the resolver would be fast enough to let us remove this check. But right now, for large environments,
    // it's an order of magnitude faster to validate the environment than to resolve the requirements.
    if reinstall.is_none()
        && upgrade.is_none()
        && source_trees.is_empty()
        && groups.is_empty()
        && pylock.is_none()
        && matches!(modifications, Modifications::Sufficient)
    {
        match site_packages.satisfies_spec(&requirements, &constraints, &overrides, &marker_env)? {
            // If the requirements are already satisfied, we're done.
            SatisfiesResult::Fresh {
                recursive_requirements,
            } => {
                if enabled!(Level::DEBUG) {
                    for requirement in recursive_requirements
                        .iter()
                        .map(ToString::to_string)
                        .sorted()
                    {
                        debug!("Requirement satisfied: {requirement}");
                    }
                }
                DefaultInstallLogger.on_audit(requirements.len(), start, printer)?;
                if dry_run.enabled() {
                    writeln!(printer.stderr(), "Would make no changes")?;
                }

                return Ok(ExitStatus::Success);
            }
            SatisfiesResult::Unsatisfied(requirement) => {
                debug!("At least one requirement is not satisfied: {requirement}");
            }
        }
    }

    // Determine the Python requirement, if the user requested a specific version.
    let python_requirement = if let Some(python_version) = python_version.as_ref() {
        PythonRequirement::from_python_version(interpreter, python_version)
    } else {
        PythonRequirement::from_interpreter(interpreter)
    };

    // Determine the tags to use for the resolution.
    let tags = resolution_tags(
        python_version.as_ref(),
        python_platform.as_ref(),
        interpreter,
    )?;

    // Collect the set of required hashes.
    let hasher = if let Some(hash_checking) = hash_checking {
        HashStrategy::from_requirements(
            requirements
                .iter()
                .chain(overrides.iter())
                .map(|entry| (&entry.requirement, entry.hashes.as_slice())),
            constraints
                .iter()
                .map(|entry| (&entry.requirement, entry.hashes.as_slice())),
            Some(&marker_env),
            hash_checking,
        )?
    } else {
        HashStrategy::None
    };

    // Incorporate any index locations from the provided sources.
    let index_locations = index_locations.combine(
        extra_index_urls
            .into_iter()
            .map(Index::from_extra_index_url)
            .chain(index_url.map(Index::from_index_url))
            .map(|index| index.with_origin(Origin::RequirementsTxt))
            .collect(),
        find_links
            .into_iter()
            .map(Index::from_find_links)
            .map(|index| index.with_origin(Origin::RequirementsTxt))
            .collect(),
        no_index,
    );

    index_locations.cache_index_credentials();

    // Determine the PyTorch backend.
    let torch_backend = torch_backend
        .map(|mode| {
            TorchStrategy::from_mode(
                mode,
                python_platform
                    .map(TargetTriple::platform)
                    .as_ref()
                    .unwrap_or(interpreter.platform())
                    .os(),
            )
        })
        .transpose()?;

    // Initialize the registry client.
    let client = RegistryClientBuilder::try_from(client_builder)?
        .cache(cache.clone())
        .index_locations(&index_locations)
        .index_strategy(index_strategy)
        .torch_backend(torch_backend.clone())
        .markers(interpreter.markers())
        .platform(interpreter.platform())
        .build();

    // Combine the `--no-binary` and `--no-build` flags from the requirements files.
    let build_options = build_options.combine(no_binary, no_build);

    // Resolve the flat indexes from `--find-links`.
    let flat_index = {
        let client = FlatIndexClient::new(client.cached_client(), client.connectivity(), &cache);
        let entries = client
            .fetch_all(index_locations.flat_indexes().map(Index::url))
            .await?;
        FlatIndex::from_entries(entries, Some(&tags), &hasher, &build_options)
    };

    // Determine whether to enable build isolation.
    let build_isolation = if no_build_isolation {
        BuildIsolation::Shared(&environment)
    } else if no_build_isolation_package.is_empty() {
        BuildIsolation::Isolated
    } else {
        BuildIsolation::SharedPackage(&environment, &no_build_isolation_package)
    };

    // Enforce (but never require) the build constraints, if `--require-hashes` or `--verify-hashes`
    // is provided. _Requiring_ hashes would be too strict, and would break with pip.
    let build_hasher = if hash_checking.is_some() {
        HashStrategy::from_requirements(
            std::iter::empty(),
            build_constraints
                .iter()
                .map(|entry| (&entry.requirement, entry.hashes.as_slice())),
            Some(&marker_env),
            HashCheckingMode::Verify,
        )?
    } else {
        HashStrategy::None
    };
    let build_constraints = Constraints::from_requirements(
        build_constraints
            .iter()
            .map(|constraint| constraint.requirement.clone()),
    );

    // Initialize any shared state.
    let state = SharedState::default();

    // Create a build dispatch.
    let extra_build_requires =
        uv_distribution::ExtraBuildRequires::from_lowered(extra_build_dependencies.clone());
    let build_dispatch = BuildDispatch::new(
        &client,
        &cache,
        build_constraints,
        interpreter,
        &index_locations,
        &flat_index,
        &dependency_metadata,
        state.clone(),
        index_strategy,
        config_settings,
        config_settings_package,
        build_isolation,
        &extra_build_requires,
        link_mode,
        &build_options,
        &build_hasher,
        exclude_newer.clone(),
        sources,
        WorkspaceCache::default(),
        concurrency,
        preview,
    );

    let (resolution, hasher) = if let Some(pylock) = pylock {
        // Read the `pylock.toml` from disk, and deserialize it from TOML.
        let install_path = std::path::absolute(&pylock)?;
        let install_path = install_path.parent().unwrap();
        let content = fs_err::tokio::read_to_string(&pylock).await?;
        let lock = toml::from_str::<PylockToml>(&content).with_context(|| {
            format!("Not a valid `pylock.toml` file: {}", pylock.user_display())
        })?;

        // Verify that the Python version is compatible with the lock file.
        if let Some(requires_python) = lock.requires_python.as_ref() {
            if !requires_python.contains(interpreter.python_version()) {
                return Err(anyhow::anyhow!(
                    "The requested interpreter resolved to Python {}, which is incompatible with the `pylock.toml`'s Python requirement: `{}`",
                    interpreter.python_version(),
                    requires_python,
                ));
            }
        }

        // Convert the extras and groups specifications into a concrete form.
        let extras = extras.with_defaults(DefaultExtras::default());
        let extras = extras
            .extra_names(lock.extras.iter())
            .cloned()
            .collect::<Vec<_>>();

        let groups = groups
            .get(&pylock)
            .cloned()
            .unwrap_or_default()
            .with_defaults(DefaultGroups::List(lock.default_groups.clone()));
        let groups = groups
            .group_names(lock.dependency_groups.iter())
            .cloned()
            .collect::<Vec<_>>();

        let resolution = lock.to_resolution(
            install_path,
            marker_env.markers(),
            &extras,
            &groups,
            &tags,
            &build_options,
        )?;
        let hasher = HashStrategy::from_resolution(&resolution, HashCheckingMode::Verify)?;

        (resolution, hasher)
    } else {
        // When resolving, don't take any external preferences into account.
        let preferences = Vec::default();

        let options = OptionsBuilder::new()
            .resolution_mode(resolution_mode)
            .prerelease_mode(prerelease_mode)
            .dependency_mode(dependency_mode)
            .exclude_newer(exclude_newer)
            .index_strategy(index_strategy)
            .torch_backend(torch_backend)
            .build_options(build_options.clone())
            .build();

        // Resolve the requirements.
        let resolution = match operations::resolve(
            requirements,
            constraints,
            overrides,
            source_trees,
            project,
            BTreeSet::default(),
            extras,
            &groups,
            preferences,
            site_packages.clone(),
            &hasher,
            &reinstall,
            &upgrade,
            Some(&tags),
            ResolverEnvironment::specific(marker_env.clone()),
            python_requirement,
            interpreter.markers(),
            Conflicts::empty(),
            &client,
            &flat_index,
            state.index(),
            &build_dispatch,
            concurrency,
            options,
            Box::new(DefaultResolveLogger),
            printer,
        )
        .await
        {
            Ok(graph) => Resolution::from(graph),
            Err(err) => {
                return diagnostics::OperationDiagnostic::native_tls(network_settings.native_tls)
                    .report(err)
                    .map_or(Ok(ExitStatus::Failure), |err| Err(err.into()));
            }
        };

        (resolution, hasher)
    };

    // Sync the environment.
    match operations::install(
        &resolution,
        site_packages,
        modifications,
        &reinstall,
        &build_options,
        link_mode,
        compile,
        &index_locations,
        config_settings,
        config_settings_package,
        &hasher,
        &tags,
        &client,
        state.in_flight(),
        concurrency,
        &build_dispatch,
        &cache,
        &environment,
        Box::new(DefaultInstallLogger),
        installer_metadata,
        dry_run,
        printer,
    )
    .await
    {
        Ok(..) => {}
        Err(err) => {
            return diagnostics::OperationDiagnostic::native_tls(network_settings.native_tls)
                .report(err)
                .map_or(Ok(ExitStatus::Failure), |err| Err(err.into()));
        }
    }

    // Notify the user of any resolution diagnostics.
    operations::diagnose_resolution(resolution.diagnostics(), printer)?;

    // Notify the user of any environment diagnostics.
    if strict && !dry_run.enabled() {
        operations::diagnose_environment(&resolution, &environment, &marker_env, printer)?;
    }

    Ok(ExitStatus::Success)
}
