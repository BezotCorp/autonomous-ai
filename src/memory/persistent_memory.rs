use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    consumables::resource_state::ResourceState,
    memory::{
        acquisition_cache::AcquisitionCache, experience_memory::ExperienceMemory,
        knowledge_memory::KnowledgeMemory, routing_memory::RoutingMemory,
    },
};

/// Everything the intelligence keeps between executions, stored in
/// `memory.bin`. The three cognitive memories are logically independent;
/// beside them are kept the last known state of the consumable resources
/// and the technical store of acquired data.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct PersistentMemory {
    knowledge: KnowledgeMemory,
    routing: RoutingMemory,
    experiences: ExperienceMemory,
    resources: BTreeMap<String, ResourceState>,
    acquisitions: AcquisitionCache,
}

impl PersistentMemory {
    pub(crate) fn resources(&self) -> &BTreeMap<String, ResourceState> {
        &self.resources
    }

    pub(crate) fn acquisitions(&self) -> &AcquisitionCache {
        &self.acquisitions
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
        self.routing = RoutingMemory::rebuild(&self.experiences);
    }
}
