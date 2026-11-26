use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::memory::activity::Activity;

/// Where an activity came from.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ActivityOrigin {
    /// A received stimulus gave rise to it.
    Stimulus(Uuid),
    /// It arose from the internal dynamics alone.
    Spontaneous,
    /// A concern gave rise to it.
    Concern(Uuid),
    /// Another activity drifted into it.
    Transformation(Activity),
    /// A conscious activity deliberately took it up (by its identifier).
    Voluntary(Uuid),
    /// An experimenter started it (laboratory).
    Intervention,
}

impl ActivityOrigin {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Stimulus(_) => "stimulus",
            Self::Spontaneous => "spontaneous",
            Self::Concern(_) => "concern",
            Self::Transformation(_) => "transformation",
            Self::Voluntary(_) => "voluntary",
            Self::Intervention => "intervention",
        }
    }
}
