use std::collections::BTreeSet;

use crate::{
    cognition::module_descriptor::ModuleDescriptor,
    memory::{
        impasse_kind::ImpasseKind, impasse_record::ImpasseRecord,
        invocation_record::InvocationRecord,
    },
    reasoning::{node_id::NodeId, reasoning_graph::ReasoningGraph},
    representation::content_kind::ContentKind,
};

/// Identifies why the needs still open at the end of an episode could not
/// be satisfied, from the graph and the invocation trace only.
///
/// Generic modules (deduction, recall) consider every need; when they
/// neither satisfied a need nor decomposed it into sub-needs and no module
/// declares that kind of information, nothing available can provide it.
pub(crate) struct ImpasseDiagnosis;

impl ImpasseDiagnosis {
    pub(crate) fn diagnose(
        graph: &ReasoningGraph,
        descriptors: &[&ModuleDescriptor],
        invocations: &[InvocationRecord],
    ) -> Vec<ImpasseRecord> {
        let open: Vec<NodeId> = graph
            .valid_nodes()
            .filter(|node| {
                node.content()
                    .need()
                    .is_some_and(|pattern| !graph.answers(pattern))
            })
            .map(|node| node.id())
            .collect();

        let open_set: BTreeSet<NodeId> = open.iter().copied().collect();
        let mut impasses = Vec::new();

        for id in open {
            let Some(node) = graph.node(id) else {
                continue;
            };
            let Some(pattern) = node.content().need() else {
                continue;
            };

            let waiting: Vec<String> = graph
                .dependents(id)
                .iter()
                .filter(|dependent| open_set.contains(dependent))
                .filter_map(|dependent| graph.node(*dependent))
                .map(|dependent| dependent.content().to_string())
                .collect();

            let kind = if !waiting.is_empty() {
                ImpasseKind::WaitingOn { needs: waiting }
            } else {
                let signature = pattern.signature();

                let suppliers: Vec<&str> = descriptors
                    .iter()
                    .copied()
                    .filter(|descriptor| {
                        descriptor.consumes(ContentKind::Need) && descriptor.declares(&signature)
                    })
                    .map(|descriptor| descriptor.name())
                    .collect();

                let attempts: Vec<&InvocationRecord> = invocations
                    .iter()
                    .filter(|invocation| {
                        invocation.step() >= node.step() && suppliers.contains(&invocation.module())
                    })
                    .collect();

                let names = |records: &[&InvocationRecord]| {
                    let names: BTreeSet<String> = records
                        .iter()
                        .map(|record| record.module().to_owned())
                        .collect();
                    names.into_iter().collect::<Vec<_>>()
                };

                let failed: Vec<&InvocationRecord> = attempts
                    .iter()
                    .copied()
                    .filter(|record| record.error().is_some())
                    .collect();

                if suppliers.is_empty() {
                    ImpasseKind::NoCapableModule
                } else if attempts.is_empty() {
                    ImpasseKind::NotAttempted {
                        modules: suppliers.iter().map(|name| (*name).to_owned()).collect(),
                    }
                } else if failed.len() == attempts.len() {
                    ImpasseKind::SuppliersFailed {
                        modules: names(&failed),
                    }
                } else {
                    ImpasseKind::SuppliersWithoutResult {
                        modules: names(&attempts),
                    }
                }
            };

            impasses.push(ImpasseRecord::new(pattern.clone(), kind));
        }

        impasses
    }
}
