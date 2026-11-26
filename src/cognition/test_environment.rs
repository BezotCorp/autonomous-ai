use std::collections::BTreeMap;

use crate::{
    acquisition::external_access::ExternalAccess,
    cognition::{
        capability_model::CapabilityModel, module_descriptor::ModuleDescriptor,
        module_environment::ModuleEnvironment,
    },
    consumables::resource_meter::ResourceMeter,
    engine::learning_parameters::LearningParameters,
    memory::{
        acquisition_cache::AcquisitionCache, knowledge_memory::KnowledgeMemory,
        persistent_memory::PersistentMemory, routing_memory::RoutingMemory,
    },
    resources::initial_resources::InitialResources,
};

/// Empty environment in which a module can be applied in isolation.
#[derive(Default)]
pub(crate) struct TestEnvironment {
    knowledge: KnowledgeMemory,
    resources: InitialResources,
    cache: AcquisitionCache,
    routing: RoutingMemory,
    parameters: LearningParameters,
    memory: PersistentMemory,
    descriptors: Vec<ModuleDescriptor>,
}

impl TestEnvironment {
    /// Environment in which these capabilities are known to exist.
    pub(crate) fn knowing(descriptors: Vec<ModuleDescriptor>) -> Self {
        Self {
            descriptors,
            ..Self::default()
        }
    }

    pub(crate) fn with<R>(&self, action: impl FnOnce(&ModuleEnvironment<'_>) -> R) -> R {
        let access = ExternalAccess::new(ResourceMeter::default(), &self.cache);
        let capabilities = CapabilityModel::new(
            &self.routing,
            &self.parameters,
            self.descriptors.clone(),
            BTreeMap::new(),
        );
        let environment = ModuleEnvironment::new(
            &self.knowledge,
            &self.resources,
            &access,
            &capabilities,
            &self.parameters,
            &self.memory,
        );

        action(&environment)
    }
}
