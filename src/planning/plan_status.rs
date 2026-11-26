use serde::{Deserialize, Serialize};

/// How the execution of an adopted plan ended.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PlanStatus {
    /// Still being followed.
    Active,
    /// Every step was carried out.
    Completed,
    /// Replaced by a revised plan after new information appeared.
    Revised,
    /// The episode ended before every step was carried out.
    Abandoned,
}
