use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::memory::{dream_mode::DreamMode, dream_variant::DreamVariant};

/// Internal experiment designed to resolve an uncertainty about the
/// cognition: replay a past experience in some mode and variant.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct ExperimentRequest {
    episode: Uuid,
    mode: DreamMode,
    variant: Option<DreamVariant>,
    reason: String,
}

impl ExperimentRequest {
    pub(crate) fn new(
        episode: Uuid,
        mode: DreamMode,
        variant: Option<DreamVariant>,
        reason: String,
    ) -> Self {
        Self {
            episode,
            mode,
            variant,
            reason,
        }
    }

    pub(crate) fn episode(&self) -> Uuid {
        self.episode
    }

    pub(crate) fn mode(&self) -> DreamMode {
        self.mode
    }

    pub(crate) fn variant(&self) -> Option<&DreamVariant> {
        self.variant.as_ref()
    }

    pub(crate) fn reason(&self) -> &str {
        &self.reason
    }

    /// Same experiment, whatever the reason given.
    pub(crate) fn same_as(&self, other: &Self) -> bool {
        self.episode == other.episode && self.mode == other.mode && self.variant == other.variant
    }
}
