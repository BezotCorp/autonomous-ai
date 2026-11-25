use serde::{Deserialize, Serialize};

use crate::repository::Repository;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct SnapshotIdentity {
    repository: Repository,
    revision: String,
}

impl SnapshotIdentity {
    pub(crate) fn revision(&self) -> &str {
        &self.revision
    }

    pub(crate) fn new(repository: Repository, revision: String) -> Self {
        Self {
            repository,
            revision,
        }
    }

    pub(crate) fn repository(&self) -> &Repository {
        &self.repository
    }
}
