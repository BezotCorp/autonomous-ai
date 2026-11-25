use serde::{Deserialize, Serialize};

use crate::{
    observation_failure::ObservationFailure, repository::Repository,
    repository_snapshot::RepositorySnapshot,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ObservationPayload {
    RepositoriesDiscovered { repositories: Vec<Repository> },
    RepositoryObserved { snapshot: RepositorySnapshot },
    Failed { failure: ObservationFailure },
}
