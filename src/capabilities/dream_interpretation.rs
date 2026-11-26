use std::collections::BTreeSet;

use crate::{
    cognition::{
        cognitive_module::CognitiveModule, module_context::ModuleContext,
        module_descriptor::ModuleDescriptor, module_output::ModuleOutput, proposal::Proposal,
    },
    influences::{influence::Influence, influence_kind::InfluenceKind},
    mathematics::random_source::RandomSource,
    memory::{
        dream_record::DreamRecord, dream_verdict::DreamVerdict, persistent_memory::PersistentMemory,
    },
    reasoning::epistemic_kind::EpistemicKind,
    representation::{
        content::Content, content_kind::ContentKind, rule::Rule, statement::Statement, term::Term,
    },
};

/// Draws real possibilities from dreams. A dream proves nothing; what it
/// suggests is confronted with what is actually known, and only then
/// offered to the conscious as an influence, whose worth will be judged
/// by its real consequences:
///
/// - a conjecture imagined in a dream that the remembered facts confirm
///   and never contradict becomes a conjecture for the conscious;
/// - a need that emerged in a dream and that a real capability could
///   satisfy becomes a potential need where its kind of information is at
///   stake; one nothing real could satisfy is reported as a gap;
/// - a capability imagined in a scene whose questions it answered is
///   reported as a capability worth having;
/// - a strategy that reached its goals in a dream, and capabilities a
///   replay showed unnecessary, weigh on the corresponding choices.
///
/// Only a few unexamined dreams are examined at a time, chosen at random:
/// no dream has to be examined, none is ever discarded.
///
/// Answers `possibility(kind, subject)`.
pub(crate) struct DreamInterpretation {
    descriptor: ModuleDescriptor,
}

impl DreamInterpretation {
    const DREAMS_PER_EXAMINATION: usize = 5;
    const MAXIMUM_MATCHES: usize = 10_000;

    pub(crate) fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "dream_interpretation",
                "Confronts what dreams suggested with reality and offers the conscious real possibilities.",
            )
            .consuming(&[ContentKind::Need])
            .supplying(&["possibility/2"])
            .producing(&[EpistemicKind::Computation])
            .reflective(),
        }
    }

    fn possibility(output: &mut ModuleOutput, kind: &str, subject: impl Into<String>) {
        output.propose(Proposal::new(
            Content::Statement(Statement::new(
                "possibility",
                vec![Term::symbol(kind), Term::text(subject.into())],
            )),
            EpistemicKind::Computation,
            Vec::new(),
        ));
    }

    /// Predicates of the real problems a kind of information is part of.
    fn situations(memory: &PersistentMemory, predicate: &str) -> BTreeSet<String> {
        memory
            .real_episodes()
            .filter(|episode| {
                episode
                    .questions()
                    .iter()
                    .any(|question| question.predicate() == predicate)
                    || episode
                        .impasses()
                        .iter()
                        .any(|impasse| impasse.need().predicate() == predicate)
            })
            .flat_map(|episode| {
                episode
                    .questions()
                    .iter()
                    .map(|question| question.predicate().to_owned())
            })
            .collect()
    }

    fn interpret(output: &mut ModuleOutput, context: &ModuleContext<'_>, dream: &DreamRecord) {
        let memory = context.memory();
        let knowledge = memory.knowledge();
        let capabilities = context.capabilities();
        let origin = vec![format!("dream:{}", dream.id())];

        let conjectures: Vec<Rule> = dream
            .hypotheses()
            .iter()
            .cloned()
            .chain(dream.scenes().iter().flat_map(|scene| {
                scene
                    .elements()
                    .iter()
                    .filter(|element| element.kind() == EpistemicKind::Imagined)
                    .filter_map(|element| element.content().rule().cloned())
                    .collect::<Vec<_>>()
            }))
            .collect();

        // What the dream itself used: a conjecture drawn from these facts is
        // not confirmed by finding them again.
        let used: BTreeSet<&Statement> = dream
            .scenes()
            .iter()
            .flat_map(|scene| scene.elements())
            .filter_map(|element| element.content().statement())
            .collect();

        for rule in conjectures {
            let (confirmations, counterexamples) = knowledge.confront(&rule, Self::MAXIMUM_MATCHES);
            let confirmations = confirmations
                .iter()
                .filter(|facts| facts.iter().all(|fact| !used.contains(fact)))
                .count();

            if confirmations > 0 && counterexamples.is_empty() {
                Self::possibility(output, "real_conjecture", rule.to_string());
                output.influence(Influence::new(
                    InfluenceKind::Conjecture { rule: rule.clone() },
                    BTreeSet::new(),
                    origin.clone(),
                    format!(
                        "imagined in a dream; {confirmations} remembered confirmations independent of the dream, no counterexample"
                    ),
                ));
            }
        }

        for scene in dream.scenes() {
            for need in scene.emerged_needs() {
                let signature = need.signature();

                if capabilities.can_obtain(&signature) {
                    let situations = Self::situations(memory, need.predicate());

                    if !situations.is_empty() {
                        Self::possibility(output, "anticipated_need", signature.clone());
                        output.influence(Influence::new(
                            InfluenceKind::PotentialNeed {
                                pattern: need.clone(),
                            },
                            situations,
                            origin.clone(),
                            "needed in a dream, and obtainable in reality".to_owned(),
                        ));
                    }
                } else {
                    Self::possibility(output, "capability_gap", signature);
                }
            }

            let answered: BTreeSet<String> = scene
                .consequences()
                .iter()
                .map(Statement::signature)
                .collect();

            for capability in scene.capabilities() {
                if answered.contains(capability.supplies()) {
                    Self::possibility(
                        output,
                        "useful_capability",
                        capability.supplies().to_owned(),
                    );
                }
            }

            if !scene.consequences().is_empty()
                && let Some(episode) = scene.episode()
            {
                let situation: BTreeSet<String> = episode
                    .questions()
                    .iter()
                    .map(|question| question.predicate().to_owned())
                    .collect();

                for record in episode.plans().iter().filter(|record| record.achieved()) {
                    for step in record.plan().steps() {
                        if !step.module().starts_with("imagined:") && !situation.is_empty() {
                            output.influence(Influence::new(
                                InfluenceKind::Favour {
                                    module: step.module().to_owned(),
                                },
                                situation.clone(),
                                origin.clone(),
                                format!(
                                    "part of a strategy that reached its goals in a dream: {}",
                                    record.plan().strategy()
                                ),
                            ));
                        }
                    }
                }
            }

            if let (
                Some((before, after, DreamVerdict::Cheaper | DreamVerdict::Equivalent)),
                Some(original),
            ) = (
                scene.comparison(),
                scene
                    .origin()
                    .and_then(|origin| memory.experiences().find(origin)),
            ) {
                let situation: BTreeSet<String> = original
                    .questions()
                    .iter()
                    .map(|question| question.predicate().to_owned())
                    .collect();

                for module in before.modules().filter(|module| !after.uses(module)) {
                    Self::possibility(output, "unnecessary_operation", module.clone());

                    if !situation.is_empty() {
                        output.influence(Influence::new(
                            InfluenceKind::Question {
                                module: module.clone(),
                            },
                            situation.clone(),
                            origin.clone(),
                            format!("the same answers were reached without {module} in a replay"),
                        ));
                    }
                }
            }
        }

        output.examined(dream.id());
    }
}

impl CognitiveModule for DreamInterpretation {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let mut output = ModuleOutput::default();

        if context.open_needs(&self.descriptor).is_empty() {
            return Ok(output);
        }

        let dreams = context.memory().dreams();
        let mut unexamined: Vec<&DreamRecord> = dreams
            .dreams()
            .iter()
            .filter(|dream| !dreams.is_examined(dream.id()))
            .collect();
        let mut random = RandomSource::new(context.graph().revision() ^ unexamined.len() as u64);

        for _ in 0..Self::DREAMS_PER_EXAMINATION {
            if unexamined.is_empty() {
                break;
            }

            let index =
                ((random.uniform() * unexamined.len() as f64) as usize).min(unexamined.len() - 1);
            let dream = unexamined.swap_remove(index);

            Self::interpret(&mut output, context, dream);
        }

        Ok(output)
    }
}
