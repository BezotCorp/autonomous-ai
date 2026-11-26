use std::collections::BTreeSet;

use crate::{
    cognition::{
        cognitive_module::CognitiveModule, module_context::ModuleContext,
        module_descriptor::ModuleDescriptor, module_output::ModuleOutput, proposal::Proposal,
    },
    memory::{
        correction::Correction, correction_basis::CorrectionBasis,
        hypothesis_status::HypothesisStatus, knowledge_memory::KnowledgeMemory,
        knowledge_origin::KnowledgeOrigin, standing::Standing,
    },
    reasoning::epistemic_kind::EpistemicKind,
    representation::{
        bindings::Bindings, content::Content, content_kind::ContentKind, rule::Rule,
        statement::Statement, term::Term,
    },
};

/// Examines what the intelligence believes and on what grounds.
///
/// - searches the remembered facts for counterexamples to open hypotheses
///   (a proof: the hypothesis is refuted);
/// - tells independent corroborations (distinct sources or scopes) from
///   repetitions of the same observation;
/// - finds contradictory beliefs, keeps the most recently supported one
///   and asks for a verification in the environment;
/// - finds derived beliefs resting on withdrawn ones;
/// - questions its own past corrections whose grounds disappeared.
///
/// Answers `knowledge_concern(kind, subject)`.
pub(crate) struct EvidenceReview {
    descriptor: ModuleDescriptor,
}

impl EvidenceReview {
    /// Bound on the partial matches explored per hypothesis.
    const MAXIMUM_MATCHES: usize = 10_000;
    /// Bound on verifications requested in one examination.
    const MAXIMUM_VERIFICATIONS: usize = 10;

    pub(crate) fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "evidence_review",
                "Examines beliefs and conjectures against the evidence the memory holds.",
            )
            .consuming(&[ContentKind::Need])
            .supplying(&["knowledge_concern/2"])
            .producing(&[EpistemicKind::Computation, EpistemicKind::Need])
            .reflective(),
        }
    }

    fn concern(output: &mut ModuleOutput, kind: &str, subject: impl Into<String>) {
        output.propose(Proposal::new(
            Content::Statement(Statement::new(
                "knowledge_concern",
                vec![Term::symbol(kind), Term::text(subject.into())],
            )),
            EpistemicKind::Computation,
            Vec::new(),
        ));
    }

    /// Facts satisfying every premise of the rule whose conclusion is
    /// known to be false: a counterexample.
    fn counterexample(knowledge: &KnowledgeMemory, rule: &Rule) -> Option<Vec<Statement>> {
        let mut partial: Vec<(Bindings, Vec<Statement>)> = vec![(Bindings::default(), Vec::new())];
        let mut explored = 0;

        for premise in rule.premises() {
            let mut next = Vec::new();

            for (bindings, facts) in &partial {
                let pattern = bindings.apply(premise);

                for entry in knowledge.recall(&pattern) {
                    explored += 1;

                    if explored > Self::MAXIMUM_MATCHES {
                        return None;
                    }

                    if let Some(extended) = bindings.unify(premise, entry.statement()) {
                        let mut facts = facts.clone();

                        facts.push(entry.statement().clone());
                        next.push((extended, facts));
                    }
                }
            }

            partial = next;
        }

        partial.into_iter().find_map(|(bindings, mut facts)| {
            let conclusion = bindings.apply(rule.conclusion());

            if !conclusion.is_ground() {
                return None;
            }

            let negation = conclusion.complement();

            knowledge
                .entry(&negation)
                .filter(|entry| entry.standing().is_active())
                .map(|_| {
                    facts.push(negation);
                    facts
                })
        })
    }

    /// Distinct observed scopes among the facts corroborating a hypothesis,
    /// and the number of corroborations.
    fn independence(
        knowledge: &KnowledgeMemory,
        corroborations: &BTreeSet<Statement>,
    ) -> (usize, usize) {
        let mut scopes = BTreeSet::new();

        for statement in corroborations {
            for evidence in knowledge
                .entry(statement)
                .into_iter()
                .flat_map(|entry| entry.evidence())
            {
                scopes.insert(evidence.provenance().map(|provenance| {
                    (
                        provenance.source().to_owned(),
                        provenance.scope().map(str::to_owned),
                    )
                }));
            }
        }

        (scopes.len(), corroborations.len())
    }
}

impl CognitiveModule for EvidenceReview {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let mut output = ModuleOutput::default();

        if context.open_needs(&self.descriptor).is_empty() {
            return Ok(output);
        }

        let memory = context.memory();
        let knowledge = memory.knowledge();

        for record in knowledge.hypotheses() {
            let rule = record.rule();

            if record.status() != &HypothesisStatus::Open {
                continue;
            }

            if let Some(facts) = Self::counterexample(knowledge, rule) {
                Self::concern(&mut output, "counterexample", rule.to_string());
                output.correct(
                    Correction::RefuteHypothesis { rule: rule.clone() },
                    CorrectionBasis::Proof {
                        facts: facts.clone(),
                    },
                    format!(
                        "the remembered facts {} contradict it",
                        facts
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                );
                continue;
            }

            let (independent, corroborations) =
                Self::independence(knowledge, record.corroborations());

            if corroborations > independent && independent <= 1 {
                Self::concern(
                    &mut output,
                    "repeated_support",
                    format!("{rule}: {corroborations} corroborations from {independent} source"),
                );
            }

            if record.origin() == KnowledgeOrigin::Dream && corroborations == 0 {
                Self::concern(&mut output, "unverified_dream_hypothesis", rule.to_string());
            }
        }

        let mut verifications = 0;

        for entry in knowledge.active_entries() {
            let statement = entry.statement();

            // Contradictions: each pair is examined once, from its
            // positive side.
            if !statement.is_negated()
                && let Some(contrary) = knowledge
                    .entry(&statement.complement())
                    .filter(|contrary| contrary.standing().is_active())
            {
                Self::concern(&mut output, "contradiction", statement.to_string());

                let latest = |entry: &crate::memory::knowledge_entry::KnowledgeEntry| {
                    entry
                        .evidence()
                        .iter()
                        .map(|evidence| evidence.recorded_at())
                        .max()
                };

                let older = match latest(entry).cmp(&latest(contrary)) {
                    std::cmp::Ordering::Less => Some(statement.clone()),
                    std::cmp::Ordering::Greater => Some(contrary.statement().clone()),
                    std::cmp::Ordering::Equal => None,
                };

                if let Some(older) = older {
                    output.correct(
                        Correction::ObsoleteFact {
                            statement: older.clone(),
                        },
                        CorrectionBasis::Inconsistency {
                            detail: format!("{statement} and its negation were both held"),
                        },
                        format!("{older} is the less recently supported side of a contradiction"),
                    );
                }

                if verifications < Self::MAXIMUM_VERIFICATIONS {
                    verifications += 1;
                    output.propose(Proposal::new(
                        Content::Need(statement.clone()),
                        EpistemicKind::Need,
                        Vec::new(),
                    ));
                }
            }

            // Derived beliefs whose every justification uses a withdrawn
            // belief no longer have grounds.
            if matches!(entry.standing(), Standing::Derived)
                && !entry.evidence().is_empty()
                && entry.evidence().iter().all(|evidence| {
                    evidence.premises().iter().any(|premise| {
                        knowledge
                            .entry(premise)
                            .is_some_and(|premise| !premise.standing().is_active())
                    })
                })
            {
                Self::concern(&mut output, "unfounded_conclusion", statement.to_string());
                output.correct(
                    Correction::ObsoleteFact {
                        statement: statement.clone(),
                    },
                    CorrectionBasis::Inconsistency {
                        detail: "every justification rests on a withdrawn belief".to_owned(),
                    },
                    format!("{statement} was derived from beliefs since withdrawn"),
                );
            }
        }

        // The examination questions its own past conclusions.
        for record in memory.corrections().records() {
            if !record.is_active() {
                continue;
            }

            if let (Correction::RefuteHypothesis { rule }, CorrectionBasis::Proof { facts }) =
                (record.correction(), record.basis())
                && facts.iter().any(|fact| {
                    knowledge
                        .entry(fact)
                        .is_none_or(|entry| !entry.standing().is_active())
                })
            {
                Self::concern(&mut output, "unfounded_correction", rule.to_string());
                output.correct(
                    Correction::ReopenHypothesis { rule: rule.clone() },
                    CorrectionBasis::Inconsistency {
                        detail: format!("grounds of correction {} withdrawn", record.id()),
                    },
                    "the facts that refuted it are no longer believed".to_owned(),
                );
            }
        }

        if verifications > 0 {
            output.note(format!(
                "{verifications} contradictory beliefs submitted for verification"
            ));
        }

        Ok(output)
    }
}
