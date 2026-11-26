use crate::{
    cognition::module_descriptor::ModuleDescriptor,
    memory::knowledge_memory::KnowledgeMemory,
    reasoning::{node_id::NodeId, reasoning_graph::ReasoningGraph},
    representation::statement::Statement,
    resources::initial_resources::InitialResources,
};

/// Read-only view given to a module when it is invoked.
pub(crate) struct ModuleContext<'a> {
    graph: &'a ReasoningGraph,
    knowledge: &'a KnowledgeMemory,
    resources: &'a InitialResources,
    seen_revision: u64,
}

impl<'a> ModuleContext<'a> {
    pub(crate) fn new(
        graph: &'a ReasoningGraph,
        knowledge: &'a KnowledgeMemory,
        resources: &'a InitialResources,
        seen_revision: u64,
    ) -> Self {
        Self {
            graph,
            knowledge,
            resources,
            seen_revision,
        }
    }

    pub(crate) fn graph(&self) -> &'a ReasoningGraph {
        self.graph
    }

    pub(crate) fn knowledge(&self) -> &'a KnowledgeMemory {
        self.knowledge
    }

    pub(crate) fn resources(&self) -> &'a InitialResources {
        self.resources
    }

    /// Graph revision up to which this module has already examined the
    /// results; newer nodes are those it has not processed yet.
    pub(crate) fn seen_revision(&self) -> u64 {
        self.seen_revision
    }

    /// Needs not processed yet by the module, still unanswered, whose
    /// signature the module declares it can supply.
    pub(crate) fn open_needs(&self, descriptor: &ModuleDescriptor) -> Vec<(NodeId, &'a Statement)> {
        self.graph
            .valid_nodes()
            .filter(|node| node.revision() > self.seen_revision)
            .filter_map(|node| Some((node.id(), node.content().need()?)))
            .filter(|(_, pattern)| {
                descriptor.supplies(&pattern.signature()) && !self.graph.answers(pattern)
            })
            .collect()
    }
}
