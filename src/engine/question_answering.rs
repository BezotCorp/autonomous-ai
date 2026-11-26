use std::collections::BTreeSet;

use crate::{
    memory::{answer_record::AnswerRecord, answer_status::AnswerStatus},
    reasoning::{epistemic_kind::EpistemicKind, node_id::NodeId, reasoning_graph::ReasoningGraph},
    representation::{bindings::Bindings, statement::Statement},
};

/// Builds the answer to a question from the valid results of the graph,
/// distinguishing proof, proof of absence and absence of proof.
pub(crate) struct QuestionAnswering;

impl QuestionAnswering {
    /// Returns the answer and the set of nodes supporting it.
    pub(crate) fn answer(
        graph: &ReasoningGraph,
        question: &Statement,
    ) -> (AnswerRecord, BTreeSet<NodeId>) {
        let bindings = Bindings::default();

        let positives: Vec<NodeId> = graph
            .matching(question, &bindings)
            .map(|(id, _)| id)
            .collect();
        let complement = question.complement();
        let negatives: Vec<NodeId> = graph
            .matching(&complement, &bindings)
            .map(|(id, _)| id)
            .collect();

        let ground = question.is_ground();

        let (status, answering) = match (positives.is_empty(), negatives.is_empty()) {
            (false, false) if ground => {
                (AnswerStatus::Contradictory, [positives, negatives].concat())
            }
            (false, _) => (AnswerStatus::Proven, positives),
            (true, false) if ground => (AnswerStatus::Refuted, negatives),
            _ => (AnswerStatus::Undetermined, Vec::new()),
        };

        let support = graph.ancestors(&answering);

        let mut answers: Vec<Statement> = answering
            .iter()
            .filter_map(|id| graph.node(*id)?.content().statement().cloned())
            .collect();

        answers.sort();
        answers.dedup();

        let justification = support
            .iter()
            .filter_map(|id| graph.node(*id))
            .map(|node| {
                let premises = node
                    .premises()
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");

                format!(
                    "{} [{:?} by {}] {} <- [{premises}]",
                    node.id(),
                    node.kind(),
                    node.producer(),
                    node.content(),
                )
            })
            .collect();

        let hypotheses = support
            .iter()
            .filter_map(|id| graph.node(*id))
            .filter(|node| node.kind() == EpistemicKind::Hypothesis)
            .map(|node| node.content().to_string())
            .collect();

        // Answering statements are as outdated as their supports; the answer
        // is outdated as soon as one of them is.
        let staleness = 1.0
            - answering
                .iter()
                .filter_map(|id| graph.node(*id))
                .map(|node| 1.0 - node.staleness())
                .product::<f64>();

        let recollections: Vec<_> = support
            .iter()
            .filter_map(|id| graph.node(*id))
            .filter(|node| node.kind() == EpistemicKind::Recollection && !node.is_current())
            .collect();

        let as_of = recollections
            .iter()
            .filter_map(|node| Some(node.provenance()?.acquired_at()))
            .min();

        (
            AnswerRecord::new(question.clone(), status, answers, justification, hypotheses).dated(
                staleness,
                as_of,
                recollections.len(),
            ),
            support,
        )
    }
}
