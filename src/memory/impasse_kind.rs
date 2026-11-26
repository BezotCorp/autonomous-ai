use serde::{Deserialize, Serialize};

/// Why a need remained unsatisfied at the end of an episode.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ImpasseKind {
    /// It waits for other needs, formulated for it, that are still open:
    /// the blockage lies further down.
    WaitingOn { needs: Vec<String> },
    /// No registered module declares that it can supply this kind of need.
    NoCapableModule,
    /// Capable modules exist but were never invoked after the need appeared.
    NotAttempted { modules: Vec<String> },
    /// Capable modules were invoked and failed technically.
    SuppliersFailed { modules: Vec<String> },
    /// Capable modules were invoked without satisfying it.
    SuppliersWithoutResult { modules: Vec<String> },
}

impl ImpasseKind {
    /// Short name used to aggregate impasses across episodes.
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::WaitingOn { .. } => "waiting_on",
            Self::NoCapableModule => "no_capable_module",
            Self::NotAttempted { .. } => "not_attempted",
            Self::SuppliersFailed { .. } => "suppliers_failed",
            Self::SuppliersWithoutResult { .. } => "suppliers_without_result",
        }
    }

    /// Whether the cause of the blockage lies in this need itself rather
    /// than in a need it depends on.
    pub(crate) fn is_root(&self) -> bool {
        !matches!(self, Self::WaitingOn { .. })
    }
}
