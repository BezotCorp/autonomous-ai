use serde::{Deserialize, Serialize};

use crate::{cargo_manifest::CargoManifest, repository::Repository};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct RepositorySnapshot {
    repository: Repository,
    revision: String,
    manifests: Vec<CargoManifest>,
    lockfiles: Vec<(String, String)>,
}

impl RepositorySnapshot {
    pub(crate) fn new(
        repository: Repository,
        revision: String,
        manifests: Vec<CargoManifest>,
        lockfiles: Vec<(String, String)>,
    ) -> Self {
        Self {
            repository,
            revision,
            manifests,
            lockfiles,
        }
    }
}
