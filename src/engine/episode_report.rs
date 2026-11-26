use serde::Serialize;

use crate::{
    engine::consolidation_summary::ConsolidationSummary, memory::episode_record::EpisodeRecord,
    reasoning::working_memory::WorkingMemory,
};

/// Observable account of an episode: the persisted experience, how the
/// knowledge evolved and, on request, the whole reasoning graph.
#[derive(Debug, Serialize)]
pub(crate) struct EpisodeReport {
    episode: EpisodeRecord,
    knowledge_changes: ConsolidationSummary,
    routing_episodes_learned: usize,
    reasoning: Option<WorkingMemory>,
}

impl EpisodeReport {
    pub(crate) fn new(
        episode: EpisodeRecord,
        knowledge_changes: ConsolidationSummary,
        routing_episodes_learned: usize,
        reasoning: Option<WorkingMemory>,
    ) -> Self {
        Self {
            episode,
            knowledge_changes,
            routing_episodes_learned,
            reasoning,
        }
    }

    #[cfg(test)]
    pub(crate) fn episode_answers(&self) -> &[crate::memory::answer_record::AnswerRecord] {
        self.episode.answers()
    }

    #[cfg(test)]
    pub(crate) fn episode_revisions(&self) -> &[crate::memory::revision_record::RevisionRecord] {
        self.episode.revisions()
    }
}
