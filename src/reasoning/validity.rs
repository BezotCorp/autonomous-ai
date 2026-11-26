use serde::Serialize;

use crate::reasoning::invalidation_cause::InvalidationCause;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Validity {
    Valid,
    Invalidated {
        cause: InvalidationCause,
        step: usize,
    },
}

impl Validity {
    pub(crate) fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }
}
