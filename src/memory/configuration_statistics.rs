use serde::Serialize;

use crate::memory::{answer_status::AnswerStatus, episode_record::EpisodeRecord};

/// Aggregated results of the episodes run under one experimental
/// configuration, to compare experiments with and without acquired
/// knowledge or learned routing.
#[derive(Debug, Default, Serialize)]
pub(crate) struct ConfigurationStatistics {
    episodes: usize,
    invocations: usize,
    proven_or_refuted: usize,
    undetermined: usize,
    revisions: usize,
    total_seconds: f64,
}

impl ConfigurationStatistics {
    pub(crate) fn label(episode: &EpisodeRecord) -> String {
        episode.configuration().label()
    }

    pub(crate) fn observe(&mut self, episode: &EpisodeRecord) {
        self.episodes += 1;
        self.invocations += episode.invocations().len();
        self.revisions += episode.revisions().len();
        self.total_seconds += episode.duration_micros() as f64 / 1e6;

        for answer in episode.answers() {
            match answer.status() {
                AnswerStatus::Proven | AnswerStatus::Refuted => self.proven_or_refuted += 1,
                AnswerStatus::Undetermined | AnswerStatus::Contradictory => self.undetermined += 1,
            }
        }
    }
}
