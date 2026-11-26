use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{memory::dream_scene::DreamScene, representation::rule::Rule};

/// One dream: why it began (a requested experiment, or nothing in
/// particular), its successive scenes, the conjectures that arose in it,
/// and what it actually cost. Every dream is kept, whatever it produced;
/// only the detailed traces of old dreams are dropped. Nothing in it is a
/// fact about the world.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct DreamRecord {
    id: Uuid,
    started_at: DateTime<Utc>,
    reason: String,
    scenes: Vec<DreamScene>,
    hypotheses: Vec<Rule>,
    seconds: f64,
}

impl DreamRecord {
    pub(crate) fn new(
        started_at: DateTime<Utc>,
        reason: String,
        scenes: Vec<DreamScene>,
        hypotheses: Vec<Rule>,
        seconds: f64,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            started_at,
            reason,
            scenes,
            hypotheses,
            seconds,
        }
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn started_at(&self) -> DateTime<Utc> {
        self.started_at
    }

    pub(crate) fn reason(&self) -> &str {
        &self.reason
    }

    pub(crate) fn scenes(&self) -> &[DreamScene] {
        &self.scenes
    }

    pub(crate) fn hypotheses(&self) -> &[Rule] {
        &self.hypotheses
    }

    pub(crate) fn seconds(&self) -> f64 {
        self.seconds
    }

    /// Scenes replaying a real experience, with their comparison.
    pub(crate) fn replays(&self) -> impl Iterator<Item = &DreamScene> {
        self.scenes
            .iter()
            .filter(|scene| scene.comparison().is_some())
    }

    /// Whether a scene replayed this experience in this mode and variant.
    pub(crate) fn replayed(
        &self,
        episode: Uuid,
        mode: crate::memory::dream_mode::DreamMode,
        variant: Option<&crate::memory::dream_variant::DreamVariant>,
    ) -> bool {
        self.replays().any(|scene| {
            scene.origin() == Some(episode)
                && scene.mode() == Some(mode)
                && scene.variant() == variant
        })
    }

    pub(crate) fn forget_traces(&mut self) {
        for scene in &mut self.scenes {
            scene.forget_trace();
        }
    }
}
