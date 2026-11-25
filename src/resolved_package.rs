use serde::{Deserialize, Serialize};

use crate::registry::Registry;

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
    pub(crate) fn version(&self) -> &semver::Version {
        &self.version
    }

    pub(crate) fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub(crate) fn is_older_than(
        &self,
        name: &str,
        registry: &Registry,
        version: &semver::Version,
    ) -> bool {
        if self.name != name {
            return false;
        }

        let Some(source) = self.source() else {
            return false;
        };

        Registry::from_source(source).as_ref() == Some(registry)
            && self.version.cmp_precedence(version).is_lt()
    }
}
