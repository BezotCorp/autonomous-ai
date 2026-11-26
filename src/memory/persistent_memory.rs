use serde::{Deserialize, Serialize};

use crate::memory::{
    experience_memory::ExperienceMemory, knowledge_memory::KnowledgeMemory,
    routing_memory::RoutingMemory,
};

/// Everything the intelligence keeps between executions, stored in
/// `memory.bin`. The three memories are logically independent.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct PersistentMemory {
    knowledge: KnowledgeMemory,
    routing: RoutingMemory,
    experiences: ExperienceMemory,
}

impl PersistentMemory {
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
