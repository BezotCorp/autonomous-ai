use std::fmt;

use serde::{Deserialize, Serialize};

/// Component that introduced a result into the reasoning graph.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Producer {
    Perception,
    Resource(String),
    Module(String),
    /// Injected by the unconscious (an influence, by its identifier).
    Unconscious(String),
    /// Introduced by the imagination during a dream.
    Imagination(String),
}

impl fmt::Display for Producer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Perception => write!(formatter, "perception"),
            Self::Resource(name) => write!(formatter, "resource:{name}"),
            Self::Module(name) => write!(formatter, "module:{name}"),
            Self::Unconscious(name) => write!(formatter, "unconscious:{name}"),
            Self::Imagination(name) => write!(formatter, "imagination:{name}"),
        }
    }
}
