use serde::{Deserialize, Serialize};

/// How a past experience is replayed. The two are never mixed in results.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DreamMode {
    /// With only the knowledge, learning and data available at the time.
    Historical,
    /// With everything known now, on the data available at the time.
    Reinterpretation,
}
