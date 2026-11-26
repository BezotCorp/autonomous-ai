use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ConcernStatus {
    Active,
    /// What it was about was answered.
    Resolved,
    /// Its salience faded away, or it was displaced by more salient ones.
    Faded,
}
