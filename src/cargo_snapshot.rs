use serde::{Deserialize, Serialize};

use crate::{cargo_lockfile::CargoLockfile, cargo_manifest::CargoManifest};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct CargoSnapshot {
    manifests: Vec<CargoManifest>,
    lockfiles: Vec<CargoLockfile>,
}

impl CargoSnapshot {
    pub(crate) fn new(manifests: Vec<CargoManifest>, lockfiles: Vec<CargoLockfile>) -> Self {
        Self {
            manifests,
            lockfiles,
        }
    }

    pub(crate) fn lockfiles(&self) -> &[CargoLockfile] {
        &self.lockfiles
    }
}
