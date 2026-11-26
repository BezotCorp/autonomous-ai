use crate::{
    acquisition::external_access::ExternalAccess, cognition::capability_model::CapabilityModel,
    engine::learning_parameters::LearningParameters, memory::knowledge_memory::KnowledgeMemory,
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
}

impl<'a> ModuleEnvironment<'a> {
    pub(crate) fn new(
        knowledge: &'a KnowledgeMemory,
        resources: &'a InitialResources,
        access: &'a ExternalAccess<'a>,
        capabilities: &'a CapabilityModel<'a>,
        parameters: &'a LearningParameters,
    ) -> Self {
        Self {
            knowledge,
            resources,
            access,
            capabilities,
            parameters,
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

    pub(crate) fn parameters(&self) -> &'a LearningParameters {
        self.parameters
    }
}
