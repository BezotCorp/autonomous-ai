use serde::{Deserialize, Serialize};

/// Whether something learned comes from acting in the real environment or
/// from an internal simulation (a dream). Simulated learning is kept
/// apart and never counts as an observation of the world.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum KnowledgeOrigin {
    #[default]
    Experience,
    Dream,
}
