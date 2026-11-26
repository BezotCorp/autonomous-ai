use std::collections::BTreeSet;

use crate::{
    memory::revision_record::RevisionRecord,
    reasoning::{
        contradiction::Contradiction, invalidation_cause::InvalidationCause, node_id::NodeId,
        working_memory::WorkingMemory,
    },
};

/// Resolves contradictions by tracing them back through the reasoning graph.
///
/// Every defeasible result (hypothesis or recollection) supporting either
/// side is a possible cause, however old it is. Withdrawing any single one
/// of them removes one side of the contradiction; the causes whose
/// withdrawal destroys the fewest other valid results are withdrawn
/// (principle of minimal change). Ties are all withdrawn rather than
/// arbitrarily broken. When no defeasible cause exists, both sides are
/// marked contested and the intelligence remains uncertain.
pub(crate) struct BeliefRevision;

impl BeliefRevision {
    pub(crate) fn resolve(
        memory: &mut WorkingMemory,
        positive: NodeId,
        negative: NodeId,
        step: usize,
    ) -> Option<(RevisionRecord, Vec<NodeId>)> {
        let contradiction = Contradiction::new(positive, negative, step);

        if !memory.graph().is_standing(&contradiction) {
            return None;
        }

        let index = memory.register(contradiction);
        let graph = memory.graph();

        let candidates: BTreeSet<NodeId> = graph
            .ancestors(&[positive, negative])
            .into_iter()
            .filter(|id| {
                graph
                    .node(*id)
                    .is_some_and(|node| node.is_valid() && node.kind().is_defeasible())
            })
            .collect();

        let describe = |id: NodeId| {
            graph
                .node(id)
                .map(|node| format!("{id} {:?} {}", node.kind(), node.content()))
                .unwrap_or_default()
        };

        let positive_text = describe(positive);
        let negative_text = describe(negative);

        if candidates.is_empty() {
            let mut invalidated = 0;

            for side in [positive, negative] {
                invalidated += memory
                    .graph_mut()
                    .invalidate(
                        side,
                        InvalidationCause::Contested {
                            contradiction: index,
                        },
                        step,
                    )
                    .len();
            }

            return Some((
                RevisionRecord::new(
                    step,
                    positive_text,
                    negative_text,
                    Vec::new(),
                    Vec::new(),
                    invalidated,
                    false,
                ),
                Vec::new(),
            ));
        }

        let damages: Vec<(NodeId, usize)> = candidates
            .iter()
            .map(|candidate| (*candidate, graph.descendants(*candidate).len()))
            .collect();

        let minimum = damages.iter().map(|(_, damage)| *damage).min().unwrap_or(0);

        let examined = damages
            .iter()
            .map(|(id, damage)| format!("{} (would invalidate {damage})", describe(*id)))
            .collect();

        let withdrawn: Vec<NodeId> = damages
            .iter()
            .filter(|(_, damage)| *damage == minimum)
            .map(|(id, _)| *id)
            .collect();

        let withdrawn_text = withdrawn.iter().map(|id| describe(*id)).collect();

        let mut invalidated = 0;

        for id in &withdrawn {
            invalidated += memory
                .graph_mut()
                .invalidate(
                    *id,
                    InvalidationCause::Retracted {
                        contradiction: index,
                    },
                    step,
                )
                .len();
        }

        Some((
            RevisionRecord::new(
                step,
                positive_text,
                negative_text,
                examined,
                withdrawn_text,
                invalidated,
                true,
            ),
            withdrawn,
        ))
    }
}
