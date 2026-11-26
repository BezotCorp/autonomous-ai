use serde::{Deserialize, Serialize};

use crate::memory::episode_record::EpisodeRecord;

/// Persistent register of every reasoning episode.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct ExperienceMemory {
    episodes: Vec<EpisodeRecord>,
}

impl ExperienceMemory {
    pub(crate) fn record(&mut self, episode: EpisodeRecord) {
        self.episodes.push(episode);
    }

    /// Experiences that had started by an instant.
    pub(crate) fn until(&self, instant: chrono::DateTime<chrono::Utc>) -> Self {
        Self {
            episodes: self
                .episodes
                .iter()
                .filter(|episode| episode.started_at() < instant)
                .cloned()
                .collect(),
        }
    }

    pub(crate) fn find(&self, id: uuid::Uuid) -> Option<&EpisodeRecord> {
        self.episodes.iter().find(|episode| episode.id() == id)
    }

    pub(crate) fn episodes(&self) -> &[EpisodeRecord] {
        &self.episodes
    }
}
