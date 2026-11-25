use serde::{Deserialize, Serialize};

use crate::{
    observation_failure::ObservationFailure, repository::Repository,
    snapshot_identity::SnapshotIdentity,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ObservationPayload {
    RepositoriesDiscovered {
        repositories: Vec<Repository>,
    },
    RepositoryObserved {
        identity: SnapshotIdentity,
        reused: bool,
    },
    Failed {
        failure: ObservationFailure,
    },
}
