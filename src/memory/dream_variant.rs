use std::fmt;

use serde::{Deserialize, Serialize};

/// Alternative explored by a dream, beside the replay itself.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DreamVariant {
    /// Another sequence of decisions (another exploratory draw).
    Reorder { seed: u64 },
    /// The same problem without one capability, to see whether it was
    /// needed at all.
    Ablation { module: String },
    /// The same problem under a simulated limit on a resource.
    Scarcity { resource: String, units: u64 },
}

impl fmt::Display for DreamVariant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reorder { seed } => write!(formatter, "reorder({seed})"),
            Self::Ablation { module } => write!(formatter, "without({module})"),
            Self::Scarcity { resource, units } => write!(formatter, "scarce({resource}={units})"),
        }
    }
}
