use serde::{Deserialize, Serialize};

/// Current standing of a persisted piece of knowledge.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Standing {
    /// Directly observed from a source.
    Observed,
    /// Deduced from established knowledge, without relying on hypotheses.
    Derived,
    /// No longer observed in the latest revision of its source, or
    /// corrected; `since` dates the change so past states can be rebuilt.
    Obsolete {
        reason: String,
        since: chrono::DateTime<chrono::Utc>,
    },
}

impl Standing {
    pub(crate) fn is_active(&self) -> bool {
        !matches!(self, Self::Obsolete { .. })
    }
}
