use serde::{Deserialize, Serialize};

use crate::{capability::Capability, repository::Repository};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ObservationFailure {
    capability: Capability,
    repository: Repository,
    error: String,
}

impl ObservationFailure {
    pub(crate) fn new(capability: Capability, repository: Repository, error: String) -> Self {
        Self {
            capability,
            repository,
            error,
        }
    }
}
