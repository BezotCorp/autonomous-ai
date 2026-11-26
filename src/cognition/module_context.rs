use crate::{
    acquisition::external_access::ExternalAccess,
    cognition::{
        capability_model::CapabilityModel, module_descriptor::ModuleDescriptor,
        module_environment::ModuleEnvironment,
    },
    engine::learning_parameters::LearningParameters,
    memory::knowledge_memory::KnowledgeMemory,
    planning::plan::Plan,
    reasoning::{epistemic_kind::EpistemicKind, node_id::NodeId, reasoning_graph::ReasoningGraph},
    representation::statement::Statement,
    resources::initial_resources::InitialResources,
};

/// Read-only view given to a module when it is invoked.
pub(crate) struct ModuleContext<'a> {
    graph: &'a ReasoningGraph,
    environment: &'a ModuleEnvironment<'a>,
    seen_revision: u64,
    active_plan: Option<&'a Plan>,
}

impl<'a> ModuleContext<'a> {
    pub(crate) fn new(
        graph: &'a ReasoningGraph,
        environment: &'a ModuleEnvironment<'a>,
        seen_revision: u64,
        active_plan: Option<&'a Plan>,
    ) -> Self {
        Self {
            graph,
            environment,
            seen_revision,
            active_plan,
        }
    }

    pub(crate) fn graph(&self) -> &'a ReasoningGraph {
        self.graph
    }

    pub(crate) fn knowledge(&self) -> &'a KnowledgeMemory {
        self.environment.knowledge()
    }

    pub(crate) fn resources(&self) -> &'a InitialResources {
        self.environment.resources()
    }

    pub(crate) fn access(&self) -> &'a ExternalAccess<'a> {
        self.environment.access()
    }

    pub(crate) fn capabilities(&self) -> &'a CapabilityModel<'a> {
        self.environment.capabilities()
    }

    pub(crate) fn memory(&self) -> &'a crate::memory::persistent_memory::PersistentMemory {
        self.environment.memory()
    }

    pub(crate) fn parameters(&self) -> &'a LearningParameters {
        self.environment.parameters()
    }

    /// Plan currently followed, if any.
    pub(crate) fn active_plan(&self) -> Option<&'a Plan> {
        self.active_plan
    }

    /// Graph revision up to which this module has already examined the
    /// results; newer nodes are those it has not processed yet.
    pub(crate) fn seen_revision(&self) -> u64 {
        self.seen_revision
    }

    /// Whether a need still calls for this module. A need answered only by
    /// recollections remains open for the modules able to observe the
    /// present state, which may verify it.
    pub(crate) fn is_open(&self, pattern: &Statement, descriptor: &ModuleDescriptor) -> bool {
        Self::open_for(self.graph, pattern, descriptor)
    }

    pub(crate) fn open_for(
        graph: &ReasoningGraph,
        pattern: &Statement,
        descriptor: &ModuleDescriptor,
    ) -> bool {
        if descriptor.produces(EpistemicKind::Observation) {
            !graph.answers_currently(pattern)
        } else {
            !graph.answers(pattern)
        }
    }

    /// Needs not processed yet by the module, still open for it, whose
    /// signature the module declares it can supply.
    pub(crate) fn open_needs(&self, descriptor: &ModuleDescriptor) -> Vec<(NodeId, &'a Statement)> {
        self.graph
            .valid_nodes()
            .filter(|node| node.revision() > self.seen_revision)
            .filter_map(|node| Some((node.id(), node.content().need()?)))
            .filter(|(_, pattern)| {
                descriptor.supplies(&pattern.signature()) && self.is_open(pattern, descriptor)
            })
            .collect()
    }
}
