use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use uv_normalize::{ExtraName, PackageName};
use uv_pep440::{Version, VersionSpecifiers};
use uv_pep508::Requirement;
use uv_pypi_types::{ResolutionMetadata, VerbatimParsedUrl};

/// Pre-defined [`StaticMetadata`] entries, indexed by [`PackageName`] and [`Version`].
#[derive(Debug, Clone, Default)]
pub struct DependencyMetadata(FxHashMap<PackageName, Vec<StaticMetadata>>);

impl DependencyMetadata {
    /// Index a set of [`StaticMetadata`] entries by [`PackageName`] and [`Version`].
    pub fn from_entries(entries: impl IntoIterator<Item = StaticMetadata>) -> Self {
        let mut map = Self::default();
        for entry in entries {
            map.0.entry(entry.name.clone()).or_default().push(entry);
        }
        map
    }

    /// Retrieve a [`StaticMetadata`] entry by [`PackageName`] and [`Version`].
    pub fn get(
        &self,
        package: &PackageName,
        version: Option<&Version>,
    ) -> Option<ResolutionMetadata> {
        let versions = self.0.get(package)?;

        if let Some(version) = version {
            // If a specific version was requested, search for an exact match, then a global match.
            let metadata = if let Some(metadata) = versions
                .iter()
                .find(|entry| entry.version.as_ref() == Some(version))
            {
                debug!("Found dependency metadata entry for `{package}=={version}`");
                metadata
            } else if let Some(metadata) = versions.iter().find(|entry| entry.version.is_none()) {
                debug!("Found global metadata entry for `{package}`");
                metadata
            } else {
                warn!("No dependency metadata entry found for `{package}=={version}`");
                return None;
            };

            Some(ResolutionMetadata {
                name: metadata.name.clone(),
                version: version.clone(),
                requires_dist: metadata.requires_dist.clone(),
                requires_python: metadata.requires_python.clone(),
                provides_extras: metadata.provides_extras.clone(),
                dynamic: false,
            })
        } else {
            // If no version was requested (i.e., it's a direct URL dependency), allow a single
            // versioned match.
            let [metadata] = versions.as_slice() else {
                warn!("Multiple dependency metadata entries found for `{package}`");
                return None;
            };
            let Some(version) = metadata.version.clone() else {
                warn!("No version found in dependency metadata entry for `{package}`");
                return None;
            };
            debug!("Found dependency metadata entry for `{package}` (assuming: `{version}`)");

            Some(ResolutionMetadata {
                name: metadata.name.clone(),
                version,
                requires_dist: metadata.requires_dist.clone(),
                requires_python: metadata.requires_python.clone(),
                provides_extras: metadata.provides_extras.clone(),
                dynamic: false,
            })
        }
    }

    /// Retrieve all [`StaticMetadata`] entries.
    pub fn values(&self) -> impl Iterator<Item = &StaticMetadata> {
        self.0.values().flatten()
    }
}

/// A subset of the Python Package Metadata 2.3 standard as specified in
/// <https://packaging.python.org/specifications/core-metadata/>.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct StaticMetadata {
    // Mandatory fields
    pub name: PackageName,
    #[cfg_attr(
        feature = "schemars",
        schemars(
            with = "Option<String>",
            description = "PEP 440-style package version, e.g., `1.2.3`"
        )
    )]
    pub version: Option<Version>,
    // Optional fields
    #[serde(default)]
    pub requires_dist: Box<[Requirement<VerbatimParsedUrl>]>,
    #[cfg_attr(
        feature = "schemars",
        schemars(
            with = "Option<String>",
            description = "PEP 508-style Python requirement, e.g., `>=3.10`"
        )
    )]
    pub requires_python: Option<VersionSpecifiers>,
    #[serde(default)]
    pub provides_extras: Box<[ExtraName]>,
}
