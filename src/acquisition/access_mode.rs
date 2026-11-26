use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// How acquisition code may reach the environment.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AccessMode {
    /// Requests are sent to the real environment.
    Live,
    /// Nothing is sent: only data recorded by `as_of` is served, exactly as
    /// it was then; what was not recorded stays missing. Data recorded
    /// before `since` would already have been at hand then, the rest would
    /// have had to be acquired: their consumption is simulated accordingly.
    Replay {
        as_of: DateTime<Utc>,
        since: DateTime<Utc>,
    },
}
