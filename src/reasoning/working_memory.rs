use serde::Serialize;

use crate::{
    reasoning::{contradiction::Contradiction, reasoning_graph::ReasoningGraph},
    representation::statement::Statement,
};

/// Temporary memory of one reasoning episode: the questions being pursued,
/// the graph of every intermediate result (hypotheses, needs, deductions)
/// and the contradictions met. It is released when the episode ends.
#[derive(Debug, Default, Serialize)]
pub(crate) struct WorkingMemory {
    questions: Vec<Statement>,
    graph: ReasoningGraph,
    contradictions: Vec<Contradiction>,
}

impl WorkingMemory {
    pub(crate) fn new(questions: Vec<Statement>) -> Self {
        Self {
            questions,
            ..Self::default()
        }
    }

    pub(crate) fn questions(&self) -> &[Statement] {
        &self.questions
    }

    pub(crate) fn graph(&self) -> &ReasoningGraph {
        &self.graph
    }

    pub(crate) fn graph_mut(&mut self) -> &mut ReasoningGraph {
        &mut self.graph
    }

    /// Registers a contradiction and returns its index.
    pub(crate) fn register(&mut self, contradiction: Contradiction) -> usize {
        self.contradictions.push(contradiction);
        self.contradictions.len() - 1
    }
}
