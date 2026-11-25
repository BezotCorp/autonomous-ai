use serde::{Deserialize, Serialize};

use crate::repository::Repository;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EventPayload {
    RepositoryChanged {
        repository: Repository,
        revision: String,
    },
    PackagePublished {
        repository: Repository,
        package: String,
        registry: String,
        version: semver::Version,
    },
    WorkflowCompleted {
        repository: Repository,
        run_id: u64,
        revision: String,
        conclusion: String,
    },
}
