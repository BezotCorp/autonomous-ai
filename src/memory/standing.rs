use serde::{Deserialize, Serialize};

/// Current standing of a persisted piece of knowledge.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Standing {
    /// Directly observed from a source.
    Observed,
    /// Deduced from established knowledge, without relying on hypotheses.
    Derived,
    /// No longer observed in the latest revision of its source.
    Obsolete { reason: String },
}

impl Standing {
    pub(crate) fn is_active(&self) -> bool {
        !matches!(self, Self::Obsolete { .. })
    }
}
