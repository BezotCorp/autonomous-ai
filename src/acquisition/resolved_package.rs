use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ResolvedPackage {
    name: String,
    version: semver::Version,
    source: Option<String>,
    checksum: Option<String>,

    #[serde(default)]
    dependencies: Vec<String>,
}

impl ResolvedPackage {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn version(&self) -> &semver::Version {
        &self.version
    }

    pub(crate) fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }
}
