use std::{collections::BTreeSet, time::Instant};

use uuid::Uuid;

use crate::{
    engine::{
        dreaming::Dreaming, imagination::Imagination, intelligence::Intelligence,
        learning_parameters::LearningParameters,
    },
    events::event::Event,
    mathematics::{beta_distribution::BetaDistribution, random_source::RandomSource},
    memory::{
        activity::Activity, activity_record::ActivityRecord, dream_variant::DreamVariant,
        dream_verdict::DreamVerdict, episode_configuration::EpisodeConfiguration,
        experiment_request::ExperimentRequest, persistent_memory::PersistentMemory,
        routing_memory::RoutingMemory,
    },
    representation::{statement::Statement, term::Term},
};

/// Activities the intelligence undertakes on its own initiative, between
/// problems: examining its functioning (the unconscious) and exploring its
/// experiences internally (dreams). No fixed succession of phases is
/// imposed: at each turn, every possible activity is valued from what such
/// activities brought before in similar situations, against its cost, and
/// the intelligence stops when none seems worth it.
pub(crate) struct Autonomy<'a> {
    intelligence: &'a Intelligence,
    parameters: &'a LearningParameters,
    reflection: bool,
    dreams: bool,
    minimum_worth: f64,
}

/// One activity the intelligence could undertake now.
struct Prospect {
    activity: Activity,
    focus: String,
    features: Vec<String>,
    task: Task,
}

enum Task {
    Reflect(Statement),
    Experiment(usize),
    Spontaneous,
}

impl<'a> Autonomy<'a> {
    pub(crate) fn new(
        intelligence: &'a Intelligence,
        parameters: &'a LearningParameters,
        reflection: bool,
        dreams: bool,
        minimum_worth: f64,
    ) -> Self {
        Self {
            intelligence,
            parameters,
            reflection,
            dreams,
            minimum_worth,
        }
    }

    /// Undertakes activities until none is worth it or the budget is spent.
    pub(crate) fn run(
        &self,
        memory: &mut PersistentMemory,
        maximum: usize,
        seed: u64,
    ) -> Vec<ActivityRecord> {
        let mut random = RandomSource::new(seed);
        let mut records = Vec::new();

        for turn in 0..maximum {
            let options = self.options(memory);
            let Some(option) = self.choose(memory, options, &mut random) else {
                break;
            };

            let started = Instant::now();
            let (worthwhile, summary) = self.undertake(memory, &option.task, seed + turn as u64);
            let record = ActivityRecord::new(
                option.activity,
                option.focus,
                option.features,
                started.elapsed().as_secs_f64(),
                worthwhile,
                summary,
            );

            memory.activities_mut().record(record.clone());
            records.push(record);
        }

        records
    }

    /// Bucket of a count (0, 1, 2-3, 4-7, ...), so that situations are
    /// told apart by orders of magnitude.
    fn bucket(count: usize) -> u32 {
        usize::BITS - count.leading_zeros()
    }

    fn options(&self, memory: &PersistentMemory) -> Vec<Prospect> {
        let mut options = Vec::new();
        let actions: Vec<_> = memory
            .real_episodes()
            .filter(|episode| episode.activity() == Activity::Action)
            .collect();

        if self.reflection {
            let reflective: BTreeSet<String> = self
                .intelligence
                .descriptors()
                .filter(|descriptor| descriptor.reflects())
                .flat_map(|descriptor| descriptor.supplied().to_vec())
                .collect();

            for signature in reflective {
                let Some(question) = Self::pattern_of(&signature) else {
                    continue;
                };

                let focus = format!("reflect:{signature}");
                let last = memory
                    .activities()
                    .records()
                    .iter()
                    .rev()
                    .find(|record| record.focus() == focus)
                    .map(|record| record.started_at());
                let new_experiences = actions
                    .iter()
                    .filter(|episode| last.is_none_or(|last| episode.started_at() > last))
                    .count();
                let new_dreams = memory
                    .dreams()
                    .dreams()
                    .iter()
                    .filter(|dream| last.is_none_or(|last| dream.started_at() > last))
                    .count();

                options.push(Prospect {
                    activity: Activity::Reflection,
                    focus,
                    features: vec![
                        format!("new_experiences:{}", Self::bucket(new_experiences)),
                        format!("new_dreams:{}", Self::bucket(new_dreams)),
                    ],
                    task: Task::Reflect(question),
                });
            }
        }

        if self.dreams {
            for (index, request) in memory.dreams().pending().iter().enumerate() {
                options.push(Prospect {
                    activity: Activity::Dream,
                    focus: format!("dream:{}", Self::variant_kind(request.variant())),
                    features: vec!["requested".to_owned(), format!("mode:{:?}", request.mode())],
                    task: Task::Experiment(index),
                });
            }

            // A dream needs no problem nor request: it may begin anytime.
            let last = memory
                .dreams()
                .dreams()
                .last()
                .map(|dream| dream.started_at());
            let new_experiences = actions
                .iter()
                .filter(|episode| last.is_none_or(|last| episode.started_at() > last))
                .count();
            let unexamined = memory
                .dreams()
                .dreams()
                .iter()
                .filter(|dream| !memory.dreams().is_examined(dream.id()))
                .count();

            options.push(Prospect {
                activity: Activity::Dream,
                focus: "dream:spontaneous".to_owned(),
                features: vec![
                    format!("new_experiences:{}", Self::bucket(new_experiences)),
                    format!("unexamined_dreams:{}", Self::bucket(unexamined)),
                ],
                task: Task::Spontaneous,
            });
        }

        options
    }

    fn variant_kind(variant: Option<&DreamVariant>) -> &'static str {
        match variant {
            None => "replay",
            Some(DreamVariant::Reorder { .. }) => "reorder",
            Some(DreamVariant::Ablation { .. }) => "ablation",
            Some(DreamVariant::Scarcity { .. }) => "scarcity",
            Some(DreamVariant::WithoutInfluence { .. }) => "influence",
        }
    }

    fn pattern_of(signature: &str) -> Option<Statement> {
        let (predicate, arity) = signature.rsplit_once('/')?;
        let arity: usize = arity.parse().ok()?;

        Some(Statement::new(
            predicate,
            (0..arity)
                .map(|index| Term::Variable(format!("x{index}")))
                .collect(),
        ))
    }

    /// Thompson sampling over what each kind of activity brought in
    /// similar situations, minus its expected duration.
    fn choose(
        &self,
        memory: &PersistentMemory,
        options: Vec<Prospect>,
        random: &mut RandomSource,
    ) -> Option<Prospect> {
        let log = memory.activities();
        let mut best: Option<(f64, f64, Prospect)> = None;

        for option in options {
            let posteriors: Vec<BetaDistribution> = option
                .features
                .iter()
                .map(String::as_str)
                .chain([RoutingMemory::GLOBAL_FEATURE])
                .map(|feature| {
                    let statistic = if feature == RoutingMemory::GLOBAL_FEATURE {
                        let mut global =
                            crate::memory::routing_statistic::RoutingStatistic::default();

                        for record in log
                            .records()
                            .iter()
                            .filter(|record| record.focus() == option.focus)
                        {
                            global.observe(record.worthwhile());
                        }

                        global
                    } else {
                        log.statistic(&option.focus, feature)
                    };

                    BetaDistribution::posterior(
                        self.parameters.prior_successes(),
                        self.parameters.prior_failures(),
                        statistic.successes(),
                        statistic.failures(),
                    )
                })
                .collect();

            let count = posteriors.len() as f64;
            let cost = self.parameters.cost_weight_per_second()
                * log.cost(&option.focus).expected_seconds().unwrap_or(0.0);
            let expected =
                posteriors.iter().map(BetaDistribution::mean).sum::<f64>() / count - cost;
            let sampled = posteriors
                .iter()
                .map(|posterior| posterior.sample(random))
                .sum::<f64>()
                / count
                - cost;

            if best
                .as_ref()
                .is_none_or(|(utility, _, _)| sampled > *utility)
            {
                best = Some((sampled, expected, option));
            }
        }

        let (_, expected, option) = best?;

        (expected >= self.minimum_worth).then_some(option)
    }

    fn undertake(&self, memory: &mut PersistentMemory, task: &Task, seed: u64) -> (bool, String) {
        match task {
            Task::Reflect(question) => self.reflect(memory, question, seed),
            Task::Experiment(index) => {
                let Some(request) = memory.dreams_mut().take(*index) else {
                    return (false, "experiment no longer pending".to_owned());
                };

                self.dream(memory, Some(&request), seed)
            }
            Task::Spontaneous => self.dream(memory, None, seed),
        }
    }

    /// An episode of the ordinary engine whose question is about the
    /// cognition itself. It may acquire externally, at the usual costs.
    fn reflect(
        &self,
        memory: &mut PersistentMemory,
        question: &Statement,
        seed: u64,
    ) -> (bool, String) {
        let previous: BTreeSet<String> = memory
            .real_episodes()
            .filter(|episode| {
                episode.activity() == Activity::Reflection && episode.questions().contains(question)
            })
            .flat_map(|episode| episode.answers().iter().flat_map(|answer| answer.answers()))
            .map(ToString::to_string)
            .collect();

        let event = Event::internal(
            format!("reflection-{}", Uuid::now_v7()),
            vec![question.clone()],
        );
        let report = self.intelligence.reason(
            &event,
            memory,
            EpisodeConfiguration::new(seed, true, true)
                .with_activity(Activity::Reflection)
                .without_influences(),
            false,
        );
        let episode = report.episode();
        let concerns: Vec<String> = episode
            .answers()
            .iter()
            .flat_map(|answer| answer.answers())
            .map(ToString::to_string)
            .collect();
        let new_concerns = concerns
            .iter()
            .filter(|concern| !previous.contains(*concern))
            .count();
        let worthwhile = new_concerns > 0
            || !episode.corrections().is_empty()
            || !episode.experiments().is_empty();

        (
            worthwhile,
            format!(
                "{question}: {} concerns ({new_concerns} new), {} corrections, {} experiments requested",
                concerns.len(),
                episode.corrections().len(),
                episode.experiments().len()
            ),
        )
    }

    /// Dreams, from a requested experiment or spontaneously. Every dream is
    /// kept; for deciding when to dream, it counts as worthwhile when it
    /// brought something new (a need, a conjecture or a capability never
    /// met in earlier dreams) or an informative replay.
    fn dream(
        &self,
        memory: &mut PersistentMemory,
        request: Option<&ExperimentRequest>,
        seed: u64,
    ) -> (bool, String) {
        let dream = Imagination::new(self.intelligence, memory, seed).dream(request);
        let items = |dream: &crate::memory::dream_record::DreamRecord| -> Vec<String> {
            dream
                .scenes()
                .iter()
                .flat_map(|scene| {
                    scene
                        .emerged_needs()
                        .iter()
                        .map(|need| format!("need {}", need.signature()))
                        .chain(
                            scene
                                .capabilities()
                                .iter()
                                .map(|capability| format!("capability {}", capability.supplies())),
                        )
                        .collect::<Vec<_>>()
                })
                .chain(
                    dream
                        .hypotheses()
                        .iter()
                        .map(|rule| format!("conjecture {rule}")),
                )
                .collect()
        };
        let known: BTreeSet<String> = memory.dreams().dreams().iter().flat_map(items).collect();
        let novel = items(&dream)
            .into_iter()
            .filter(|item| !known.contains(item))
            .collect::<BTreeSet<_>>()
            .len();
        let informative = dream.replays().any(|scene| {
            matches!(
                scene.verdict(),
                Some(DreamVerdict::Cheaper | DreamVerdict::MoreAnswers)
            ) || (matches!(scene.variant(), Some(DreamVariant::Ablation { .. }))
                && scene.verdict() != Some(DreamVerdict::Failed))
        });
        let worthwhile = novel > 0 || informative;
        let summary = format!(
            "{} ({}): {} scenes [{}], {novel} new elements{}",
            dream.reason(),
            request
                .and_then(|request| request.variant().map(ToString::to_string))
                .unwrap_or_else(|| "free".to_owned()),
            dream.scenes().len(),
            dream
                .scenes()
                .iter()
                .map(|scene| format!("{}: {}", scene.transition(), scene.mechanisms().join("+")))
                .collect::<Vec<_>>()
                .join(" | "),
            dream
                .replays()
                .map(|scene| format!("; {:?} {}", scene.verdict(), scene.findings().join("; ")))
                .collect::<String>()
        );

        Dreaming::integrate(memory, dream);

        (worthwhile, summary)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::Autonomy;
    use crate::{
        capabilities::{
            dream_interpretation::DreamInterpretation, evidence_review::EvidenceReview,
            learning_review::LearningReview, logical_inference::LogicalInference,
        },
        cognition::{module_registry::ModuleRegistry, scripted_module::ScriptedModule},
        engine::{
            dreaming::Dreaming, imagination::Imagination, intelligence::Intelligence,
            learning_parameters::LearningParameters, memory_correction::MemoryCorrection,
        },
        events::event::Event,
        influences::{influence::Influence, influence_kind::InfluenceKind},
        memory::{
            activity::Activity, answer_status::AnswerStatus, correction::Correction,
            dream_mode::DreamMode, dream_record::DreamRecord, dream_variant::DreamVariant,
            dream_verdict::DreamVerdict, episode_configuration::EpisodeConfiguration,
            experiment_request::ExperimentRequest, hypothesis_status::HypothesisStatus,
            persistent_memory::PersistentMemory,
        },
        reasoning::epistemic_kind::EpistemicKind,
        resources::initial_resources::InitialResources,
    };

    fn problem(id: &str) -> Event {
        ron::de::from_str(&format!(
            r#"(id: "{id}", source: "test", occurred_at: "2026-11-26T00:00:00Z",
                payload: assertions(statements: ["context(x)"]), questions: ["fact(?which, 1)"])"#
        ))
        .unwrap()
    }

    fn intelligence(oracle: bool, reflective: bool) -> Intelligence {
        let mut registry = ModuleRegistry::default();

        registry.register(ScriptedModule::new(
            "gatherer",
            Some("item(?x)"),
            "fact(?x, 1)",
        ));
        registry.register(ScriptedModule::new("lister", None, "item(a)"));

        if oracle {
            registry.register(ScriptedModule::new("oracle", None, "fact(a, 1)"));
        }

        if reflective {
            registry.register(EvidenceReview::new());
            registry.register(LearningReview::new());
            registry.register(DreamInterpretation::new());
        }

        Intelligence::new(
            registry,
            InitialResources::default(),
            LearningParameters::default(),
        )
    }

    fn without(modules: &[&str]) -> BTreeSet<String> {
        modules.iter().map(|module| (*module).to_owned()).collect()
    }

    fn replay(
        intelligence: &Intelligence,
        memory: &PersistentMemory,
        episode: uuid::Uuid,
        mode: DreamMode,
        variant: Option<DreamVariant>,
        seed: u64,
    ) -> DreamRecord {
        Imagination::new(intelligence, memory, seed).dream(Some(&ExperimentRequest::new(
            episode,
            mode,
            variant,
            "test".to_owned(),
        )))
    }

    #[test]
    fn replays_recorded_data_and_leaves_the_rest_missing() {
        let intelligence = intelligence(false, false);
        let mut memory = PersistentMemory::default();

        intelligence.reason(
            &problem("recorded"),
            &mut memory,
            EpisodeConfiguration::new(1, true, true),
            false,
        );

        let original = memory.experiences().episodes()[0].clone();

        assert!(
            memory
                .as_of(original.started_at())
                .experiences()
                .episodes()
                .is_empty()
        );

        let dream = replay(
            &intelligence,
            &memory,
            original.id(),
            DreamMode::Historical,
            None,
            2,
        );
        let scene = dream.replays().next().unwrap();

        assert!(
            matches!(
                scene.verdict(),
                Some(DreamVerdict::Equivalent | DreamVerdict::Cheaper)
            ),
            "{dream:#?}"
        );
        assert_eq!(scene.episode().unwrap().activity(), Activity::Dream);

        // Without anything recorded, a replay cannot invent the data.
        let mut forgetful = PersistentMemory::default();

        intelligence.reason(
            &problem("unrecorded"),
            &mut forgetful,
            EpisodeConfiguration::new(3, true, true).without_acquisition_cache(),
            false,
        );

        let original = forgetful.experiences().episodes()[0].clone();
        let dream = replay(
            &intelligence,
            &forgetful,
            original.id(),
            DreamMode::Reinterpretation,
            None,
            4,
        );
        let scene = dream.replays().next().unwrap();

        assert_eq!(scene.verdict(), Some(DreamVerdict::Worse), "{dream:#?}");
        assert!(
            scene
                .findings()
                .iter()
                .any(|finding| finding.contains("left missing"))
        );
        assert_eq!(forgetful.experiences().episodes().len(), 1);
    }

    #[test]
    fn examination_refutes_then_questions_its_own_correction() {
        let intelligence = intelligence(false, true);
        let mut memory = PersistentMemory::default();
        let episode = uuid::Uuid::now_v7();
        let rule: crate::representation::rule::Rule = "mortal(?x) :- human(?x)".parse().unwrap();
        let negation: crate::representation::statement::Statement =
            "not mortal(zeus)".parse().unwrap();

        {
            let (knowledge, _, _) = memory.parts_mut();

            knowledge.record_observation("human(zeus)".parse().unwrap(), None, episode);
            knowledge.record_observation(negation.clone(), None, episode);
            knowledge.propose_hypothesis(rule.clone(), episode, 3);
        }

        let reflect = |memory: &mut PersistentMemory, seed| {
            intelligence.reason(
                &Event::internal(
                    format!("reflection-{seed}"),
                    vec!["knowledge_concern(?k, ?s)".parse().unwrap()],
                ),
                memory,
                EpisodeConfiguration::new(seed, true, true)
                    .with_activity(Activity::Reflection)
                    .without_influences(),
                false,
            )
        };

        let report = reflect(&mut memory, 1);

        assert_eq!(
            memory.knowledge().hypothesis(&rule).unwrap().status(),
            &HypothesisStatus::Refuted,
            "{report:#?}"
        );

        {
            let (knowledge, _, _) = memory.parts_mut();

            knowledge.withdraw(&negation, "turned out wrong".to_owned());
        }

        reflect(&mut memory, 2);

        assert_eq!(
            memory.knowledge().hypothesis(&rule).unwrap().status(),
            &HypothesisStatus::Open
        );
        assert!(!memory.corrections().records()[0].is_active());
    }

    #[test]
    fn corrects_attribution_only_on_accumulated_evidence_and_reversibly() {
        let intelligence = intelligence(true, true);
        let mut memory = PersistentMemory::default();

        // The gatherer and the lister record what they acquire.
        intelligence.reason(
            &problem("without-oracle"),
            &mut memory,
            EpisodeConfiguration::new(1, true, true).without_modules(without(&["oracle"])),
            false,
        );

        let reflect = |memory: &mut PersistentMemory, seed| {
            intelligence.reason(
                &Event::internal(
                    format!("reflection-{seed}"),
                    vec!["learning_concern(?k, ?s)".parse().unwrap()],
                ),
                memory,
                EpisodeConfiguration::new(seed, true, true)
                    .with_activity(Activity::Reflection)
                    .without_influences(),
                false,
            )
        };
        let reassessed = |memory: &PersistentMemory| {
            memory.corrections().records().iter().any(|record| {
                matches!(record.correction(), Correction::ReassessInvocation { module, .. } if module == "oracle")
            })
        };

        for round in 0..6_u64 {
            // The oracle alone answers, and is credited for it...
            intelligence.reason(
                &problem(&format!("oracle-{round}")),
                &mut memory,
                EpisodeConfiguration::new(10 + round, true, true)
                    .without_modules(without(&["gatherer", "lister"])),
                false,
            );

            let original = memory.experiences().episodes().last().unwrap().clone();

            // ...but the same answers are reached without it.
            let dream = replay(
                &intelligence,
                &memory,
                original.id(),
                DreamMode::Historical,
                Some(DreamVariant::Ablation {
                    module: "oracle".to_owned(),
                }),
                100 + round,
            );

            Dreaming::integrate(&mut memory, dream);
            reflect(&mut memory, 200 + round);

            if round == 0 {
                // One replay is no proof.
                assert!(!reassessed(&memory));
            }
        }

        assert!(reassessed(&memory));

        let correction = memory
            .corrections()
            .records()
            .iter()
            .find(|record| matches!(record.correction(), Correction::ReassessInvocation { .. }))
            .unwrap()
            .id();

        MemoryCorrection::revert(&mut memory, correction, "test".to_owned()).unwrap();

        assert_eq!(
            memory.corrections().reassessments().len() + 1,
            memory
                .corrections()
                .records()
                .iter()
                .filter(|record| matches!(
                    record.correction(),
                    Correction::ReassessInvocation { .. }
                ))
                .count()
        );
    }

    #[test]
    fn gives_never_tried_alternatives_a_chance() {
        let intelligence = intelligence(true, false);
        let mut memory = PersistentMemory::default();

        // Habit forms with the gatherer...
        for seed in 1..=5 {
            intelligence.reason(
                &problem(&format!("habit-{seed}")),
                &mut memory,
                EpisodeConfiguration::new(seed, true, true).without_modules(without(&["oracle"])),
                false,
            );
        }

        // ...yet the never-tried oracle still gets tried.
        let tried = (10..50).any(|seed| {
            intelligence.reason(
                &problem(&format!("p{seed}")),
                &mut memory,
                EpisodeConfiguration::new(seed, true, true),
                false,
            );

            memory
                .experiences()
                .episodes()
                .last()
                .unwrap()
                .invocations()
                .iter()
                .any(|invocation| invocation.module() == "oracle")
        });

        assert!(tried);
    }

    #[test]
    fn measures_and_retires_influences_reversibly() {
        let intelligence = intelligence(true, false);
        let mut memory = PersistentMemory::default();

        for seed in 1..=5 {
            intelligence.reason(
                &problem(&format!("habit-{seed}")),
                &mut memory,
                EpisodeConfiguration::new(seed, true, true).without_modules(without(&["oracle"])),
                false,
            );
        }

        let favour = Influence::new(
            InfluenceKind::Favour {
                module: "oracle".to_owned(),
            },
            BTreeSet::from(["fact".to_owned()]),
            vec!["test".to_owned()],
            "test".to_owned(),
        );
        let id = favour.id();

        assert!(memory.influences_mut().propose(favour));

        for seed in 10..20 {
            intelligence.reason(
                &problem(&format!("p{seed}")),
                &mut memory,
                EpisodeConfiguration::new(seed, true, true),
                false,
            );
        }

        let evidence = memory.influences().evidence()[&id];

        assert_eq!(evidence.exposures(), 10);
        assert!(
            evidence.outcomes().0 + evidence.outcomes().1 > 0,
            "{evidence:?}"
        );
        assert_eq!(evidence.attribution(), "observed_contribution");

        // The conscious alone is not influenced.
        intelligence.reason(
            &problem("alone"),
            &mut memory,
            EpisodeConfiguration::new(30, true, true).without_influences(),
            false,
        );
        assert_eq!(memory.influences().evidence()[&id].exposures(), 10);

        let correction = MemoryCorrection::apply(
            &mut memory,
            uuid::Uuid::now_v7(),
            Correction::RetireInfluence { influence: id },
            crate::memory::correction_basis::CorrectionBasis::Inconsistency {
                detail: "test".to_owned(),
            },
            "test".to_owned(),
        )
        .unwrap();

        assert!(memory.influences().active().next().is_none());
        MemoryCorrection::revert(&mut memory, correction, "test".to_owned()).unwrap();
        assert!(memory.influences().active().next().is_some());
    }

    #[test]
    fn imagines_freely_without_touching_reality() {
        let intelligence = intelligence(false, false);
        let mut memory = PersistentMemory::default();

        for seed in 1..=3 {
            intelligence.reason(
                &problem(&format!("p{seed}")),
                &mut memory,
                EpisodeConfiguration::new(seed, true, true),
                false,
            );
        }

        let facts_before = memory.knowledge().entries().count();
        let mut mechanisms = BTreeSet::new();
        let mut imagined = 0;

        for seed in 0..20 {
            let dream = Imagination::new(&intelligence, &memory, seed).dream(None);

            assert!(!dream.scenes().is_empty());

            for scene in dream.scenes() {
                mechanisms.extend(scene.mechanisms().iter().cloned());
                imagined += scene
                    .elements()
                    .iter()
                    .filter(|element| element.kind() == EpistemicKind::Imagined)
                    .count();

                if let Some(episode) = scene.episode() {
                    assert_eq!(episode.activity(), Activity::Dream);
                    assert!(matches!(
                        episode.configuration().access(),
                        crate::acquisition::access_mode::AccessMode::Replay { .. }
                    ));
                }
            }

            Dreaming::integrate(&mut memory, dream);
        }

        assert!(imagined > 0);
        assert!(mechanisms.len() >= 5, "{mechanisms:?}");
        assert_eq!(memory.dreams().dreams().len(), 20);
        // Dreams never become knowledge.
        assert_eq!(memory.knowledge().entries().count(), facts_before);
    }

    #[test]
    fn turns_a_dreamt_conjecture_confirmed_by_reality_into_an_influence() {
        let mut registry = ModuleRegistry::default();

        registry.register(LogicalInference::new());
        registry.register(DreamInterpretation::new());

        let intelligence = Intelligence::new(
            registry,
            InitialResources::default(),
            LearningParameters::default(),
        );
        let mut memory = PersistentMemory::default();
        let episode = uuid::Uuid::now_v7();

        {
            let (knowledge, _, _) = memory.parts_mut();

            for entity in ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l"] {
                knowledge.record_observation(
                    format!("bird({entity})").parse().unwrap(),
                    None,
                    episode,
                );
                knowledge.record_observation(
                    format!("flies({entity})").parse().unwrap(),
                    None,
                    episode,
                );
            }
        }

        // A dream in which flying was associated with being a bird.
        let dream = loop_until_conjecture(&intelligence, &memory);

        Dreaming::integrate(&mut memory, dream);

        intelligence.reason(
            &Event::internal(
                "reflection".to_owned(),
                vec!["possibility(?k, ?s)".parse().unwrap()],
            ),
            &mut memory,
            EpisodeConfiguration::new(1, true, true)
                .with_activity(Activity::Reflection)
                .without_influences(),
            false,
        );

        assert!(
            memory
                .influences()
                .active()
                .any(|influence| matches!(influence.kind(), InfluenceKind::Conjecture { .. }))
        );

        // The conscious, facing a new bird, can use it.
        let event: Event = ron::de::from_str(
            r#"(id: "tweety", source: "test", occurred_at: "2026-11-26T00:00:00Z",
                payload: assertions(statements: ["bird(tweety)"]), questions: ["flies(tweety)"])"#,
        )
        .unwrap();
        let report = intelligence.reason(
            &event,
            &mut memory,
            EpisodeConfiguration::new(2, true, true),
            false,
        );

        assert_eq!(
            report.episode().answers()[0].status(),
            AnswerStatus::Proven,
            "{report:#?}"
        );
        assert!(
            memory
                .influences()
                .exposures()
                .iter()
                .any(|exposure| exposure.supported_answer())
        );
    }

    /// Dreams until the imagination associates flying with being a bird.
    fn loop_until_conjecture(
        intelligence: &Intelligence,
        memory: &PersistentMemory,
    ) -> DreamRecord {
        for seed in 0..500 {
            let dream = Imagination::new(intelligence, memory, seed).dream(None);
            let associated = dream.scenes().iter().any(|scene| {
                scene.elements().iter().any(|element| {
                    element.kind() == EpistemicKind::Imagined
                        && element
                            .content()
                            .rule()
                            .is_some_and(|rule| rule.to_string() == "flies(?x) :- bird(?x)")
                })
            });

            if associated {
                return dream;
            }
        }

        panic!("the imagination never associated flying with being a bird");
    }

    #[test]
    fn undertakes_activities_on_its_own_and_knows_when_to_stop() {
        let intelligence = intelligence(false, true);
        let parameters = LearningParameters::default();
        let mut memory = PersistentMemory::default();

        for seed in 1..=3 {
            intelligence.reason(
                &problem(&format!("p{seed}")),
                &mut memory,
                EpisodeConfiguration::new(seed, true, true),
                false,
            );
        }

        assert!(
            Autonomy::new(&intelligence, &parameters, true, true, 1.01)
                .run(&mut memory, 5, 1)
                .is_empty()
        );

        let records =
            Autonomy::new(&intelligence, &parameters, true, true, 0.0).run(&mut memory, 8, 2);

        assert_eq!(records.len(), 8);
        assert!(
            records
                .iter()
                .any(|record| record.activity() == Activity::Dream)
        );
        assert!(
            records
                .iter()
                .any(|record| record.activity() == Activity::Reflection)
        );
        assert_eq!(memory.activities().records().len(), 8);
    }
}
