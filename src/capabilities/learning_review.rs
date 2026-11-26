use std::collections::{BTreeMap, BTreeSet};

use crate::{
    cognition::{
        cognitive_module::CognitiveModule, module_context::ModuleContext,
        module_descriptor::ModuleDescriptor, module_output::ModuleOutput, proposal::Proposal,
    },
    influences::{influence::Influence, influence_kind::InfluenceKind},
    mathematics::beta_distribution::BetaDistribution,
    memory::{
        activity::Activity, correction::Correction, correction_basis::CorrectionBasis,
        dream_mode::DreamMode, dream_variant::DreamVariant, dream_verdict::DreamVerdict,
        episode_record::EpisodeRecord, experiment_request::ExperimentRequest,
        invocation_outcome::InvocationOutcome, persistent_memory::PersistentMemory,
    },
    reasoning::epistemic_kind::EpistemicKind,
    representation::{
        content::Content, content_kind::ContentKind, statement::Statement, term::Term,
    },
};

/// Examines how the intelligence has been deciding and learning, on its
/// recent real experiences and on its dreams.
///
/// - capabilities solicited far more than they help, or helping far more
///   than they are solicited, judged against the others with credible
///   bounds rather than fixed rates; the doubtful ones are put to the test
///   by dreaming the same problem without them;
/// - credit wrongly attributed, revealed by such dreams: an invocation
///   credited although the same answers were reached without it, or
///   blamed although answers were lost without it, is re-assessed;
/// - recurring impasses: limitations met again and again;
/// - needs discovered late, and old problems left undetermined, to be
///   replayed with what is known now;
/// - strategies whose actual cost departs from the anticipated one;
/// - answers obtained only in simulation, to be verified in reality.
///
/// Answers `learning_concern(kind, subject)`.
pub(crate) struct LearningReview {
    descriptor: ModuleDescriptor,
}

/// Outcomes of a module over the examined experiences.
#[derive(Default)]
struct Usage {
    invocations: u64,
    successes: u64,
    episodes: BTreeSet<uuid::Uuid>,
}

impl LearningReview {
    /// Recent real experiences examined.
    const WINDOW: usize = 50;
    /// Width of the credible bounds, in standard deviations.
    const CREDIBILITY: f64 = 2.0;
    /// Bound on the simulated answers submitted for real verification.
    const MAXIMUM_VERIFICATIONS: usize = 3;

    pub(crate) fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "learning_review",
                "Examines past decisions, credit attribution and strategies against experience and dreams.",
            )
            .consuming(&[ContentKind::Need])
            .supplying(&["learning_concern/2"])
            .producing(&[EpistemicKind::Computation, EpistemicKind::Need])
            .reflective(),
        }
    }

    fn concern(output: &mut ModuleOutput, kind: &str, subject: impl Into<String>) {
        output.propose(Proposal::new(
            Content::Statement(Statement::new(
                "learning_concern",
                vec![Term::symbol(kind), Term::text(subject.into())],
            )),
            EpistemicKind::Computation,
            Vec::new(),
        ));
    }

    fn recent(memory: &PersistentMemory) -> Vec<&EpisodeRecord> {
        let mut episodes: Vec<&EpisodeRecord> = memory
            .real_episodes()
            .filter(|episode| episode.activity() == Activity::Action)
            .collect();

        episodes.reverse();
        episodes.truncate(Self::WINDOW);
        episodes
    }

    fn usage(memory: &PersistentMemory, episodes: &[&EpisodeRecord]) -> BTreeMap<String, Usage> {
        let reassessments = memory.corrections().reassessments();
        let mut usage: BTreeMap<String, Usage> = BTreeMap::new();

        for episode in episodes {
            for invocation in episode.invocations() {
                let outcome = reassessments
                    .get(&(episode.id(), invocation.step()))
                    .copied()
                    .unwrap_or(invocation.outcome());
                let entry = usage.entry(invocation.module().to_owned()).or_default();

                entry.invocations += 1;
                entry.episodes.insert(episode.id());

                if outcome.is_success(!episode.questions().is_empty()) {
                    entry.successes += 1;
                }
            }
        }

        usage
    }

    fn capabilities(
        output: &mut ModuleOutput,
        usage: &BTreeMap<String, Usage>,
        episodes: &[&EpisodeRecord],
    ) {
        let (successes, invocations) =
            usage
                .values()
                .fold((0, 0), |(successes, invocations), usage| {
                    (successes + usage.successes, invocations + usage.invocations)
                });

        if invocations == 0 {
            return;
        }

        let overall = successes as f64 / invocations as f64;

        for (module, usage) in usage {
            let posterior = BetaDistribution::posterior(
                1.0,
                1.0,
                usage.successes,
                usage.invocations - usage.successes,
            );
            let deviation = posterior.variance().sqrt() * Self::CREDIBILITY;

            if posterior.mean() + deviation < overall {
                Self::concern(
                    output,
                    "unproductive_capability",
                    format!(
                        "{module}: {} successes in {} invocations",
                        usage.successes, usage.invocations
                    ),
                );

                // Does any answer actually depend on it? Dream the most
                // recent problem it was used on without it.
                if let Some(episode) = episodes
                    .iter()
                    .find(|episode| usage.episodes.contains(&episode.id()))
                {
                    output.experiment(ExperimentRequest::new(
                        episode.id(),
                        DreamMode::Historical,
                        Some(DreamVariant::Ablation {
                            module: module.clone(),
                        }),
                        format!("is {module} needed at all?"),
                    ));
                }
            } else if posterior.mean() - deviation > overall
                && (usage.episodes.len() as f64) < episodes.len() as f64 / 4.0
            {
                Self::concern(
                    output,
                    "underused_capability",
                    format!(
                        "{module}: useful in {} of {} invocations, used in {} of {} problems",
                        usage.successes,
                        usage.invocations,
                        usage.episodes.len(),
                        episodes.len()
                    ),
                );
            }
        }
    }

    /// Credit given at the time, compared with what ablation replays
    /// showed. One replay proves little: the replays of a capability's
    /// removal accumulate as evidence over distinct problems, and credit is
    /// re-assessed only once it is more credible than not that the
    /// capability was unnecessary (or necessary); until then, further
    /// replays are requested on other problems.
    fn attribution(
        output: &mut ModuleOutput,
        memory: &PersistentMemory,
        episodes: &[&EpisodeRecord],
    ) {
        let reassessments = memory.corrections().reassessments();
        let mut evidence: BTreeMap<String, BTreeMap<uuid::Uuid, (uuid::Uuid, bool)>> =
            BTreeMap::new();

        for dream in memory.dreams().dreams() {
            for scene in dream.replays() {
                let (Some(DreamVariant::Ablation { module }), Some(origin), Some(verdict)) =
                    (scene.variant(), scene.origin(), scene.verdict())
                else {
                    continue;
                };

                let unnecessary = match verdict {
                    DreamVerdict::Cheaper | DreamVerdict::Equivalent => true,
                    DreamVerdict::Worse => false,
                    DreamVerdict::MoreAnswers | DreamVerdict::Failed => continue,
                };

                evidence
                    .entry(module.clone())
                    .or_default()
                    .insert(origin, (dream.id(), unnecessary));
            }
        }

        for (module, tested) in &evidence {
            let unnecessary = tested
                .values()
                .filter(|(_, unnecessary)| *unnecessary)
                .count() as u64;
            let necessary = tested.len() as u64 - unnecessary;
            let posterior = BetaDistribution::posterior(1.0, 1.0, unnecessary, necessary);
            let deviation = posterior.variance().sqrt() * Self::CREDIBILITY;
            let verdict = if posterior.mean() - deviation > 0.5 {
                Some(true)
            } else if posterior.mean() + deviation < 0.5 {
                Some(false)
            } else {
                None
            };

            let Some(unnecessary_credibly) = verdict else {
                Self::concern(
                    output,
                    "uncertain_attribution",
                    format!(
                        "{module}: unnecessary in {unnecessary} of {} replays",
                        tested.len()
                    ),
                );

                if let Some(episode) = episodes.iter().find(|episode| {
                    !tested.contains_key(&episode.id())
                        && episode
                            .invocations()
                            .iter()
                            .any(|invocation| invocation.module() == module)
                }) {
                    output.experiment(ExperimentRequest::new(
                        episode.id(),
                        DreamMode::Historical,
                        Some(DreamVariant::Ablation {
                            module: module.clone(),
                        }),
                        format!("more evidence on whether {module} is needed"),
                    ));
                }

                continue;
            };

            let corrected = if unnecessary_credibly {
                InvocationOutcome::Retained
            } else {
                InvocationOutcome::Advanced
            };

            for (origin, (dream, unnecessary)) in tested {
                let Some(original) = memory.experiences().find(*origin) else {
                    continue;
                };

                if *unnecessary != unnecessary_credibly {
                    continue;
                }

                let questions_asked = !original.questions().is_empty();

                for invocation in original
                    .invocations()
                    .iter()
                    .filter(|invocation| invocation.module() == module)
                {
                    let outcome = reassessments
                        .get(&(original.id(), invocation.step()))
                        .copied()
                        .unwrap_or(invocation.outcome());

                    if outcome.is_success(questions_asked) != unnecessary_credibly
                        || outcome == InvocationOutcome::Failed
                    {
                        continue;
                    }

                    let subject = format!(
                        "{module} at step {} of {}",
                        invocation.step(),
                        original.event_id()
                    );

                    Self::concern(output, "misattributed_credit", subject.clone());
                    output.correct(
                        Correction::ReassessInvocation {
                            episode: original.id(),
                            step: invocation.step(),
                            module: module.clone(),
                            outcome: corrected,
                        },
                        CorrectionBasis::Experiment { dream: *dream },
                        format!(
                            "{subject}: over {} replays without {module}, the answers were kept in {unnecessary}",
                            tested.len()
                        ),
                    );
                }
            }
        }
    }

    /// Overall rate of verified success of the examined invocations.
    fn baseline(usage: &BTreeMap<String, Usage>) -> f64 {
        let (successes, invocations) =
            usage
                .values()
                .fold((0, 0), |(successes, invocations), usage| {
                    (successes + usage.successes, invocations + usage.invocations)
                });

        (successes as f64 + 1.0) / (invocations as f64 + 2.0)
    }

    /// Alternatives starved by habit: capabilities declaring the same kind
    /// of information as a habitual one, far less tried. Their estimates
    /// are too uncertain to be trusted either way; an influence gives them
    /// a real chance in the situations concerned, which their real
    /// outcomes will then confirm or not.
    fn alternatives(
        output: &mut ModuleOutput,
        context: &ModuleContext<'_>,
        usage: &BTreeMap<String, Usage>,
        episodes: &[&EpisodeRecord],
    ) {
        let descriptors: Vec<_> = context
            .capabilities()
            .descriptors()
            .iter()
            .filter(|descriptor| !descriptor.plans() && !descriptor.reflects())
            .collect();
        let signatures: BTreeSet<&String> = descriptors
            .iter()
            .flat_map(|descriptor| descriptor.supplied())
            .collect();
        let none = Usage::default();

        for signature in signatures {
            let suppliers: Vec<&str> = descriptors
                .iter()
                .filter(|descriptor| descriptor.declares(signature))
                .map(|descriptor| descriptor.name())
                .collect();

            let Some(habitual) = suppliers
                .iter()
                .max_by_key(|module| usage.get(**module).map_or(0, |usage| usage.invocations))
                .copied()
            else {
                continue;
            };
            let habit = usage.get(habitual).unwrap_or(&none);

            if habit.invocations == 0 {
                continue;
            }

            let spread = |usage: &Usage| {
                BetaDistribution::posterior(
                    1.0,
                    1.0,
                    usage.successes,
                    usage.invocations - usage.successes,
                )
                .variance()
                .sqrt()
            };

            let situation: BTreeSet<String> = episodes
                .iter()
                .filter(|episode| habit.episodes.contains(&episode.id()))
                .flat_map(|episode| {
                    episode
                        .questions()
                        .iter()
                        .map(|question| question.predicate().to_owned())
                })
                .chain(
                    signature
                        .rsplit_once('/')
                        .map(|(predicate, _)| predicate.to_owned()),
                )
                .collect();

            for alternative in suppliers.iter().filter(|module| **module != habitual) {
                let tried = usage.get(*alternative).unwrap_or(&none);

                if tried.invocations < habit.invocations && spread(tried) > spread(habit) {
                    Self::concern(
                        output,
                        "starved_alternative",
                        format!(
                            "{alternative} supplies {signature} like {habitual}: tried {} times against {}",
                            tried.invocations, habit.invocations
                        ),
                    );
                    output.influence(Influence::new(
                        InfluenceKind::Favour {
                            module: (*alternative).to_owned(),
                        },
                        situation.clone(),
                        vec![format!("habit:{habitual}")],
                        format!(
                            "{alternative} was hardly tried where {habitual} is habitually used"
                        ),
                    ));
                }
            }
        }
    }

    /// Its own influences, judged on their real consequences: those whose
    /// changed decisions credibly fared worse than decisions in general are
    /// retired; those that changed decisions without a demonstration of
    /// their effect are put to the test by a replay without them.
    fn influences(output: &mut ModuleOutput, memory: &PersistentMemory, baseline: f64) {
        let evidence = memory.influences().evidence();

        for influence in memory.influences().active() {
            let Some(evidence) = evidence.get(&influence.id()) else {
                continue;
            };
            let (successes, failures) = evidence.outcomes();

            if successes + failures == 0 {
                continue;
            }

            let posterior = BetaDistribution::posterior(1.0, 1.0, successes, failures);
            let deviation = posterior.variance().sqrt() * Self::CREDIBILITY;

            if posterior.mean() + deviation < baseline {
                Self::concern(output, "harmful_influence", influence.kind().to_string());
                output.correct(
                    Correction::RetireInfluence {
                        influence: influence.id(),
                    },
                    CorrectionBasis::Outcomes {
                        successes,
                        failures,
                        baseline,
                    },
                    format!(
                        "the decisions it changed succeeded {successes} times in {}, below the usual rate",
                        successes + failures
                    ),
                );
                continue;
            }

            if posterior.mean() - deviation > baseline {
                Self::concern(output, "beneficial_influence", influence.kind().to_string());
            }

            if evidence.demonstrations() == (0, 0)
                && let Some(exposure) =
                    memory
                        .influences()
                        .exposures()
                        .iter()
                        .rev()
                        .find(|exposure| {
                            exposure.influence() == influence.id()
                                && !exposure.decisive().is_empty()
                        })
            {
                output.experiment(ExperimentRequest::new(
                    exposure.episode(),
                    DreamMode::Historical,
                    Some(DreamVariant::WithoutInfluence {
                        influence: influence.id(),
                    }),
                    format!("did {} make a difference?", influence.kind()),
                ));
            }
        }
    }

    fn limitations(output: &mut ModuleOutput, episodes: &[&EpisodeRecord]) {
        let mut impasses: BTreeMap<String, usize> = BTreeMap::new();

        for episode in episodes {
            let kinds: BTreeSet<String> = episode
                .impasses()
                .iter()
                .filter(|impasse| impasse.kind().is_root())
                .map(|impasse| format!("{} {}", impasse.kind().label(), impasse.need().signature()))
                .collect();

            for kind in kinds {
                *impasses.entry(kind).or_default() += 1;
            }
        }

        for (impasse, count) in impasses {
            if count * 2 >= episodes.len() && count > 1 {
                Self::concern(
                    output,
                    "recurring_impasse",
                    format!("{impasse} in {count} of {} problems", episodes.len()),
                );
            }
        }
    }

    /// Problems worth replaying with what is known now: those left
    /// undetermined, and those where a needed information was only
    /// discovered after acquisitions had started.
    fn replays(output: &mut ModuleOutput, episodes: &[&EpisodeRecord]) {
        for episode in episodes.iter().take(Self::WINDOW / 5) {
            let undetermined = episode.answers().iter().any(|answer| {
                answer.status() == crate::memory::answer_status::AnswerStatus::Undetermined
            });

            let late_need = episode.invocations().iter().skip(1).find_map(|invocation| {
                invocation
                    .produced_features()
                    .iter()
                    .find(|feature| feature.starts_with("need:"))
                    .filter(|_| invocation.outcome() == InvocationOutcome::Advanced)
                    .map(|feature| (invocation.module().to_owned(), feature.clone()))
            });

            if undetermined {
                Self::concern(output, "undetermined_problem", episode.event_id());
                output.experiment(ExperimentRequest::new(
                    episode.id(),
                    DreamMode::Reinterpretation,
                    None,
                    "re-examine an undetermined problem with what is known now".to_owned(),
                ));
            } else if let Some((module, need)) = late_need {
                Self::concern(
                    output,
                    "late_need",
                    format!(
                        "{need} formulated by {module} during {}",
                        episode.event_id()
                    ),
                );
                output.experiment(ExperimentRequest::new(
                    episode.id(),
                    DreamMode::Reinterpretation,
                    Some(DreamVariant::Reorder {
                        seed: episode.id().as_u128() as u64,
                    }),
                    format!("could {need} have been anticipated?"),
                ));
            }
        }
    }

    fn strategies(output: &mut ModuleOutput, memory: &PersistentMemory) {
        for (goal, strategies) in memory.routing().strategies() {
            for (strategy, statistic) in strategies {
                let (Some(actual), Some(predicted)) =
                    (statistic.mean_cost(), statistic.mean_predicted_cost())
                else {
                    continue;
                };

                if statistic.executions() >= 3
                    && (actual - predicted).abs() > predicted.abs().max(1e-3)
                {
                    Self::concern(
                        output,
                        "miscalibrated_strategy",
                        format!("{goal}: {strategy} costs {actual:.4}, anticipated {predicted:.4}"),
                    );
                }
            }
        }
    }

    /// Answers only reached in a dream are submitted to reality.
    fn simulated_answers(output: &mut ModuleOutput, memory: &PersistentMemory) {
        let mut submitted = 0;

        for scene in memory
            .dreams()
            .dreams()
            .iter()
            .rev()
            .flat_map(|dream| dream.replays())
        {
            if scene.verdict() != Some(DreamVerdict::MoreAnswers)
                || submitted >= Self::MAXIMUM_VERIFICATIONS
            {
                continue;
            }

            let Some(original) = scene
                .origin()
                .and_then(|origin| memory.experiences().find(origin))
            else {
                continue;
            };

            for question in original.questions() {
                Self::concern(output, "unverified_simulation", question.to_string());
                output.propose(Proposal::new(
                    Content::Need(question.clone()),
                    EpistemicKind::Need,
                    Vec::new(),
                ));
            }

            submitted += 1;
        }
    }
}

impl CognitiveModule for LearningReview {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let mut output = ModuleOutput::default();

        if context.open_needs(&self.descriptor).is_empty() {
            return Ok(output);
        }

        let memory = context.memory();
        let episodes = Self::recent(memory);
        let usage = Self::usage(memory, &episodes);

        Self::capabilities(&mut output, &usage, &episodes);
        Self::alternatives(&mut output, context, &usage, &episodes);
        Self::influences(&mut output, memory, Self::baseline(&usage));
        Self::attribution(&mut output, memory, &episodes);
        Self::limitations(&mut output, &episodes);
        Self::replays(&mut output, &episodes);
        Self::strategies(&mut output, memory);
        Self::simulated_answers(&mut output, memory);

        Ok(output)
    }
}
