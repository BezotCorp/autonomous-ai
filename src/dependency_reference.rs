use std::collections::BTreeSet;

use serde::Serialize;

use crate::repository::Repository;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DependencyReference {
    repository: Repository,
    revision: String,
    file: String,
    import_name: String,
    package: String,
    version: semver::Version,
    registry: String,
    identifiers: BTreeSet<String>,
}

impl DependencyReference {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        repository: Repository,
        revision: String,
        file: String,
        import_name: String,
        package: String,
        version: semver::Version,
        registry: String,
        identifiers: BTreeSet<String>,
    ) -> Self {
        Self {
            repository,
            revision,
            file,
            import_name,
            package,
            version,
            registry,
            identifiers,
        }
    }

    pub(crate) fn package(&self) -> &str {
        &self.package
    }

    pub(crate) fn version(&self) -> &semver::Version {
        &self.version
    }

    pub(crate) fn identifiers(&self) -> &BTreeSet<String> {
        &self.identifiers
    }
}
