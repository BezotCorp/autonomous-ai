use std::collections::{BTreeMap, BTreeSet};

use uuid::Uuid;

use crate::{
    engine::consolidation_summary::ConsolidationSummary,
    memory::knowledge_memory::KnowledgeMemory,
    reasoning::{
        epistemic_kind::EpistemicKind, node_id::NodeId, reasoning_graph::ReasoningGraph,
        reasoning_node::ReasoningNode,
    },
    representation::{bindings::Bindings, rule::Rule, statement::Statement},
};

/// Integrates the results of an episode into the persistent knowledge.
///
/// - valid observations are recorded with their provenance, and statements
///   missing from a newly observed revision of a scope become obsolete;
/// - valid deductions that rely on no hypothesis are recorded as derived;
/// - hypotheses are recorded as such, never as facts; a deduction made
///   from a hypothesis and independently observed corroborates it;
/// - hypotheses withdrawn by belief revision are recorded as refuted.
pub(crate) struct Consolidation;

impl Consolidation {
    pub(crate) fn integrate(
        graph: &ReasoningGraph,
        knowledge: &mut KnowledgeMemory,
        episode: Uuid,
        withdrawn: &[(NodeId, String)],
    ) -> ConsolidationSummary {
        let mut summary = ConsolidationSummary::default();
        let mut scopes: BTreeMap<(String, String, String), BTreeSet<Statement>> = BTreeMap::new();
        let mut verified: BTreeMap<
            (String, String),
            (Option<String>, chrono::DateTime<chrono::Utc>),
        > = BTreeMap::new();

        for node in graph.valid_nodes() {
            let Some(statement) = node.content().statement() else {
                continue;
            };

            match node.kind() {
                EpistemicKind::Observation | EpistemicKind::Replay => {
                    if let Some(provenance) = node.provenance()
                        && let Some(scope) = provenance.scope()
                    {
                        verified.insert(
                            (provenance.source().to_owned(), scope.to_owned()),
                            (
                                provenance.revision().map(str::to_owned),
                                provenance.acquired_at(),
                            ),
                        );
                    }

                    if let Some(provenance) = node.provenance()
                        && let (Some(scope), Some(revision)) =
                            (provenance.scope(), provenance.revision())
                    {
                        scopes
                            .entry((
                                provenance.source().to_owned(),
                                scope.to_owned(),
                                revision.to_owned(),
                            ))
                            .or_default()
                            .insert(statement.clone());
                    }

                    if knowledge.record_observation(
                        statement.clone(),
                        node.provenance().cloned(),
                        episode,
                    ) {
                        summary.observed();
                    }
                }

                EpistemicKind::Deduction if Self::is_established(graph, node) => {
                    let premises = node
                        .premises()
                        .iter()
                        .filter_map(|id| graph.node(*id)?.content().statement().cloned())
                        .collect();

                    if knowledge.record_derivation(statement.clone(), premises, episode) {
                        summary.derived();
                    }
                }

                _ => {}
            }
        }

        for ((source, scope), (revision, at)) in &verified {
            if knowledge.verify_scope(source, scope, revision.as_deref(), *at) {
                summary.changed_scope();
            }
        }

        for ((source, scope, revision), observed) in &scopes {
            summary.obsolete(knowledge.supersede_scope(source, scope, revision, observed));
        }

        for node in graph.valid_nodes() {
            // Conjectures injected by the unconscious stay its influences.
            if node.kind() == EpistemicKind::Hypothesis
                && !matches!(
                    node.producer(),
                    crate::reasoning::producer::Producer::Unconscious(_)
                )
                && let Some(rule) = node.content().rule()
                && knowledge.propose_hypothesis(
                    rule.clone(),
                    episode,
                    Self::support(graph, node, rule),
                )
            {
                summary.proposed(rule.clone());
            }
        }

        for node in graph.valid_nodes() {
            let (EpistemicKind::Deduction, Some(statement)) =
                (node.kind(), node.content().statement())
            else {
                continue;
            };

            let independently_observed = graph.holding(statement).any(|id| {
                graph.node(id).is_some_and(|other| {
                    matches!(
                        other.kind(),
                        EpistemicKind::Observation
                            | EpistemicKind::Replay
                            | EpistemicKind::Recollection
                    )
                })
            });

            if !independently_observed {
                continue;
            }

            for premise in node.premises().iter().filter_map(|id| graph.node(*id)) {
                if premise.kind() == EpistemicKind::Hypothesis
                    && let Some(rule) = premise.content().rule()
                    && knowledge.corroborate(rule, statement.clone())
                {
                    summary.corroborated(rule.clone(), statement.clone());
                }
            }
        }

        for (id, reason) in withdrawn {
            let Some(node) = graph.node(*id) else {
                continue;
            };

            if let Some(rule) = node.content().rule() {
                knowledge.propose_hypothesis(
                    rule.clone(),
                    episode,
                    Self::support(graph, node, rule),
                );
                knowledge.refute(rule, reason.clone());
                summary.refuted(rule.clone());
            }
        }

        summary
    }

    /// A deduction is established when none of its supports is a hypothesis.
    fn is_established(graph: &ReasoningGraph, node: &ReasoningNode) -> bool {
        graph.ancestors(&[node.id()]).iter().all(|id| {
            graph
                .node(*id)
                .is_some_and(|ancestor| ancestor.kind() != EpistemicKind::Hypothesis)
        })
    }

    /// Number of examples justifying a hypothesis: its premises that are
    /// instances of the conclusion.
    fn support(graph: &ReasoningGraph, node: &ReasoningNode, rule: &Rule) -> u64 {
        let bindings = Bindings::default();

        node.premises()
            .iter()
            .filter_map(|id| graph.node(*id)?.content().statement())
            .filter(|statement| bindings.unify(rule.conclusion(), statement).is_some())
            .count() as u64
    }
}
