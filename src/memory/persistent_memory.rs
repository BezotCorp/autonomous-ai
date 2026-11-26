use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    consumables::resource_state::ResourceState,
    memory::{
        acquisition_cache::AcquisitionCache, activity::Activity, activity_log::ActivityLog,
        correction_journal::CorrectionJournal, dream_memory::DreamMemory,
        experience_memory::ExperienceMemory, knowledge_memory::KnowledgeMemory,
        routing_memory::RoutingMemory,
    },
};

/// Everything the intelligence keeps between executions, stored in
/// `memory.bin`. The three cognitive memories are logically independent;
/// beside them are kept the last known state of the consumable resources,
/// the technical store (and record) of acquired data, the dreams, the
/// journal of corrections made by the examination of the cognition, and
/// the activities undertaken on the intelligence's own initiative.
///
/// Concurrent activities never share it: the store locks the file for the
/// whole duration of an activity.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct PersistentMemory {
    knowledge: KnowledgeMemory,
    routing: RoutingMemory,
    experiences: ExperienceMemory,
    resources: BTreeMap<String, ResourceState>,
    acquisitions: AcquisitionCache,
    dreams: DreamMemory,
    corrections: CorrectionJournal,
    activities: ActivityLog,
}

impl PersistentMemory {
    pub(crate) fn resources(&self) -> &BTreeMap<String, ResourceState> {
        &self.resources
    }

    pub(crate) fn acquisitions(&self) -> &AcquisitionCache {
        &self.acquisitions
    }

    pub(crate) fn dreams(&self) -> &DreamMemory {
        &self.dreams
    }

    pub(crate) fn dreams_mut(&mut self) -> &mut DreamMemory {
        &mut self.dreams
    }

    pub(crate) fn corrections(&self) -> &CorrectionJournal {
        &self.corrections
    }

    pub(crate) fn corrections_mut(&mut self) -> &mut CorrectionJournal {
        &mut self.corrections
    }

    pub(crate) fn activities(&self) -> &ActivityLog {
        &self.activities
    }

    pub(crate) fn activities_mut(&mut self) -> &mut ActivityLog {
        &mut self.activities
    }

    /// Isolated reconstruction of the cognitive state at an earlier
    /// instant: knowledge as it stood then, learning from the experiences
    /// lived until then, and the record of data acquired so far (which
    /// replays serve as of that instant). Nothing done in it reaches the
    /// real memory.
    pub(crate) fn as_of(&self, instant: chrono::DateTime<chrono::Utc>) -> Self {
        let experiences = self.experiences.until(instant);
        let routing =
            RoutingMemory::rebuild(&experiences, &DreamMemory::default(), &self.corrections);

        Self {
            knowledge: self.knowledge.as_of(instant),
            routing,
            experiences,
            resources: BTreeMap::new(),
            acquisitions: self.acquisitions.clone(),
            dreams: DreamMemory::default(),
            corrections: CorrectionJournal::default(),
            activities: ActivityLog::default(),
        }
    }

    /// Experiences of real activities (action and reflection).
    pub(crate) fn real_episodes(
        &self,
    ) -> impl Iterator<Item = &crate::memory::episode_record::EpisodeRecord> {
        self.experiences
            .episodes()
            .iter()
            .filter(|episode| episode.activity() != Activity::Dream)
    }

    pub(crate) fn resources_mut(&mut self) -> &mut BTreeMap<String, ResourceState> {
        &mut self.resources
    }

    pub(crate) fn acquisitions_mut(&mut self) -> &mut AcquisitionCache {
        &mut self.acquisitions
    }

    pub(crate) fn knowledge(&self) -> &KnowledgeMemory {
        &self.knowledge
    }

    pub(crate) fn routing(&self) -> &RoutingMemory {
        &self.routing
    }

    pub(crate) fn experiences(&self) -> &ExperienceMemory {
        &self.experiences
    }

    pub(crate) fn parts_mut(
        &mut self,
    ) -> (
        &mut KnowledgeMemory,
        &mut RoutingMemory,
        &mut ExperienceMemory,
    ) {
        (
            &mut self.knowledge,
            &mut self.routing,
            &mut self.experiences,
        )
    }

    /// Recomputes the routing memory from the experience register, e.g.
    /// after the learning algorithm changed.
    pub(crate) fn relearn_routing(&mut self) {
        self.routing = RoutingMemory::rebuild(&self.experiences, &self.dreams, &self.corrections);
    }
}
