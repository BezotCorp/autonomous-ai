use crate::{
    acquisition::external_access::ExternalAccess,
    cognition::capability_model::CapabilityModel,
    engine::learning_parameters::LearningParameters,
    memory::{knowledge_memory::KnowledgeMemory, persistent_memory::PersistentMemory},
    resources::initial_resources::InitialResources,
};

/// What every module may consult during an episode besides the reasoning
/// graph: the persistent knowledge, the initial resources, the access to
/// external sources, the learned model of the available capabilities and
/// the decision parameters.
pub(crate) struct ModuleEnvironment<'a> {
    knowledge: &'a KnowledgeMemory,
    resources: &'a InitialResources,
    access: &'a ExternalAccess<'a>,
    capabilities: &'a CapabilityModel<'a>,
    parameters: &'a LearningParameters,
    memory: &'a PersistentMemory,
}

impl<'a> ModuleEnvironment<'a> {
    pub(crate) fn new(
        knowledge: &'a KnowledgeMemory,
        resources: &'a InitialResources,
        access: &'a ExternalAccess<'a>,
        capabilities: &'a CapabilityModel<'a>,
        parameters: &'a LearningParameters,
        memory: &'a PersistentMemory,
    ) -> Self {
        Self {
            knowledge,
            resources,
            access,
            capabilities,
            parameters,
            memory,
        }
    }

    pub(crate) fn knowledge(&self) -> &'a KnowledgeMemory {
        self.knowledge
    }

    pub(crate) fn resources(&self) -> &'a InitialResources {
        self.resources
    }

    pub(crate) fn access(&self) -> &'a ExternalAccess<'a> {
        self.access
    }

    pub(crate) fn capabilities(&self) -> &'a CapabilityModel<'a> {
        self.capabilities
    }

    /// The whole memory of the intelligence, for examining its own
    /// functioning (experiences, learning, dreams, corrections).
    pub(crate) fn memory(&self) -> &'a PersistentMemory {
        self.memory
    }

    pub(crate) fn parameters(&self) -> &'a LearningParameters {
        self.parameters
    }
}
