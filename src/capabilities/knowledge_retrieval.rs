use std::collections::BTreeSet;

use crate::{
    cognition::{
        cognitive_module::CognitiveModule, module_context::ModuleContext,
        module_descriptor::ModuleDescriptor, module_output::ModuleOutput, proposal::Proposal,
    },
    memory::hypothesis_status::HypothesisStatus,
    reasoning::epistemic_kind::EpistemicKind,
    representation::{bindings::Bindings, content::Content, content_kind::ContentKind},
};

/// Searches the persistent knowledge for what the reasoning needs: active
/// statements matching a need (or its negation) and open hypotheses able
/// to conclude it. Recalled statements stay defeasible: a newer
/// observation may contradict them.
pub(crate) struct KnowledgeRetrieval {
    descriptor: ModuleDescriptor,
}

impl KnowledgeRetrieval {
    const MAXIMUM_RECOLLECTIONS: usize = 10_000;

    pub(crate) fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "knowledge_retrieval",
                "Recalls persisted statements and hypotheses relevant to the current needs.",
            )
            .consuming(&[ContentKind::Need])
            .producing(&[EpistemicKind::Recollection, EpistemicKind::Hypothesis]),
        }
    }
}

impl CognitiveModule for KnowledgeRetrieval {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let graph = context.graph();
        let knowledge = context.knowledge();
        let mut output = ModuleOutput::default();
        let mut recalled = BTreeSet::new();
        let mut hypotheses = BTreeSet::new();

        for node in graph.valid_nodes() {
            let Some(pattern) = node.content().need() else {
                continue;
            };

            if node.revision() <= context.seen_revision() || graph.answers(pattern) {
                continue;
            }

            let complement = pattern.complement();

            for entry in knowledge
                .recall(pattern)
                .chain(knowledge.recall(&complement))
            {
                if recalled.len() >= Self::MAXIMUM_RECOLLECTIONS {
                    output.note("recollection budget reached");
                    break;
                }

                if recalled.insert(entry.statement().clone()) {
                    let mut proposal = Proposal::new(
                        Content::Statement(entry.statement().clone()),
                        EpistemicKind::Recollection,
                        Vec::new(),
                    );

                    if let Some(provenance) = entry
                        .evidence()
                        .iter()
                        .find_map(|evidence| evidence.provenance())
                    {
                        proposal = proposal.with_provenance(provenance.clone());
                    }

                    output.propose(proposal);
                }
            }

            for record in knowledge.hypotheses() {
                if record.status() == &HypothesisStatus::Open
                    && Bindings::default()
                        .unify(record.rule().conclusion(), pattern)
                        .is_some()
                    && hypotheses.insert(record.rule().clone())
                {
                    output.note(format!(
                        "recalled hypothesis {} (support {})",
                        record.rule(),
                        record.support()
                    ));
                    output.propose(Proposal::new(
                        Content::Rule(record.rule().clone()),
                        EpistemicKind::Hypothesis,
                        Vec::new(),
                    ));
                }
            }
        }

        Ok(output)
    }
}
