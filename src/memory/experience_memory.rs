use serde::{Deserialize, Serialize};

use crate::memory::episode_record::EpisodeRecord;

/// Persistent register of every reasoning episode.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct ExperienceMemory {
    episodes: Vec<EpisodeRecord>,
}

impl ExperienceMemory {
    pub(crate) fn record(&mut self, episode: EpisodeRecord) {
        self.episodes.push(episode);
    }

    pub(crate) fn episodes(&self) -> &[EpisodeRecord] {
        &self.episodes
    }
}
