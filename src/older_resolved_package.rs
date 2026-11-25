use serde::{Deserialize, Serialize};

use crate::{resolved_package::ResolvedPackage, snapshot_identity::SnapshotIdentity};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct OlderResolvedPackage {
    snapshot: SnapshotIdentity,
    lockfile: String,
    version: semver::Version,
    source: Option<String>,
}

impl OlderResolvedPackage {
    pub(crate) fn new(
        snapshot: SnapshotIdentity,
        lockfile: String,
        package: &ResolvedPackage,
    ) -> Self {
        Self {
            snapshot,
            lockfile,
            version: package.version().clone(),
            source: package.source().map(str::to_owned),
        }
    }
}
