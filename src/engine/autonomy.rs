use std::{collections::BTreeSet, time::Instant};

use uuid::Uuid;

use crate::{
    engine::{
        dreaming::Dreaming, intelligence::Intelligence, learning_parameters::LearningParameters,
    },
    events::event::Event,
    mathematics::{beta_distribution::BetaDistribution, random_source::RandomSource},
    memory::{
        activity::Activity, activity_record::ActivityRecord, answer_status::AnswerStatus,
        dream_mode::DreamMode, dream_variant::DreamVariant, dream_verdict::DreamVerdict,
        episode_configuration::EpisodeConfiguration, experiment_request::ExperimentRequest,
        persistent_memory::PersistentMemory, routing_memory::RoutingMemory,
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
    Explore(Uuid, DreamMode),
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

            for mode in [DreamMode::Historical, DreamMode::Reinterpretation] {
                let Some(episode) =
                    actions.iter().rev().find(|episode| {
                        episode.event().is_some()
                            && !memory.dreams().dreams().iter().any(|dream| {
                                dream.original() == episode.id() && dream.mode() == mode
                            })
                    })
                else {
                    continue;
                };

                let undetermined = episode
                    .answers()
                    .iter()
                    .any(|answer| answer.status() == AnswerStatus::Undetermined);
                let useless = episode
                    .invocations()
                    .iter()
                    .filter(|invocation| !invocation.outcome().is_success(true))
                    .count();

                options.push(Prospect {
                    activity: Activity::Dream,
                    focus: format!("dream:explore:{mode:?}"),
                    features: vec![
                        format!("undetermined:{undetermined}"),
                        format!("useless_invocations:{}", Self::bucket(useless)),
                    ],
                    task: Task::Explore(episode.id(), mode),
                });
            }
        }

        options
    }

    fn variant_kind(variant: Option<&DreamVariant>) -> &'static str {
        match variant {
            None => "replay",
            Some(DreamVariant::Reorder { .. }) => "reorder",
            Some(DreamVariant::Ablation { .. }) => "ablation",
            Some(DreamVariant::Scarcity { .. }) => "scarcity",
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

                self.dream(memory, &request, seed)
            }
            Task::Explore(episode, mode) => {
                let request = ExperimentRequest::new(
                    *episode,
                    *mode,
                    None,
                    "spontaneous exploration of a past experience".to_owned(),
                );

                self.dream(memory, &request, seed)
            }
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
            EpisodeConfiguration::new(seed, true, true).with_activity(Activity::Reflection),
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

    fn dream(
        &self,
        memory: &mut PersistentMemory,
        request: &ExperimentRequest,
        seed: u64,
    ) -> (bool, String) {
        let Some(original) = memory.experiences().find(request.episode()).cloned() else {
            return (
                false,
                "the experience to replay is no longer recorded".to_owned(),
            );
        };

        let dream = Dreaming::dream(
            self.intelligence,
            memory,
            &original,
            request.mode(),
            request.variant().cloned(),
            request.reason().to_owned(),
            seed,
        );

        // A dream is worth it when it teaches something: a cheaper way, an
        // answer to verify, a capability shown unnecessary or necessary,
        // or a new conjecture.
        let ablation = matches!(request.variant(), Some(DreamVariant::Ablation { .. }));
        let worthwhile = match dream.verdict() {
            DreamVerdict::Cheaper | DreamVerdict::MoreAnswers => true,
            DreamVerdict::Equivalent | DreamVerdict::Worse => ablation,
            DreamVerdict::Failed => false,
        } || !dream.hypotheses().is_empty();

        let summary = format!(
            "{:?} {} of {}{}: {:?}; {}",
            request.mode(),
            request
                .variant()
                .map(ToString::to_string)
                .unwrap_or_else(|| "replay".to_owned()),
            original.event_id(),
            format_args!(" ({})", request.reason()),
            dream.verdict(),
            dream.findings().join("; ")
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
        capabilities::{evidence_review::EvidenceReview, learning_review::LearningReview},
        cognition::{module_registry::ModuleRegistry, scripted_module::ScriptedModule},
        engine::{
            dreaming::Dreaming, intelligence::Intelligence, learning_parameters::LearningParameters,
        },
        events::event::Event,
        memory::{
            activity::Activity, correction::Correction, dream_mode::DreamMode,
            dream_variant::DreamVariant, dream_verdict::DreamVerdict,
            episode_configuration::EpisodeConfiguration, hypothesis_status::HypothesisStatus,
            persistent_memory::PersistentMemory,
        },
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
        }

        Intelligence::new(
            registry,
            InitialResources::default(),
            LearningParameters::default(),
        )
    }

    #[test]
    fn dreams_replay_recorded_data_and_leave_the_rest_missing() {
        let intelligence = intelligence(false, false);
        let mut memory = PersistentMemory::default();

        intelligence.reason(
            &problem("recorded"),
            &mut memory,
            EpisodeConfiguration::new(1, true, true),
            false,
        );

        let original = memory.experiences().episodes()[0].clone();

        // The memory as it stood before that experience knew nothing of it.
        assert!(
            memory
                .as_of(original.started_at())
                .experiences()
                .episodes()
                .is_empty()
        );

        let dream = Dreaming::dream(
            &intelligence,
            &memory,
            &original,
            DreamMode::Historical,
            None,
            "test".to_owned(),
            2,
        );

        assert!(
            matches!(
                dream.verdict(),
                DreamVerdict::Equivalent | DreamVerdict::Cheaper
            ),
            "{dream:#?}"
        );

        let episode = dream.episode().unwrap();

        assert_eq!(episode.activity(), Activity::Dream);
        assert!(episode.usage().requests().is_empty() || episode.usage().refused() == 0);

        // Without anything recorded, a replay cannot invent the data.
        let mut forgetful = PersistentMemory::default();

        intelligence.reason(
            &problem("unrecorded"),
            &mut forgetful,
            EpisodeConfiguration::new(3, true, true).without_acquisition_cache(),
            false,
        );

        let original = forgetful.experiences().episodes()[0].clone();
        let dream = Dreaming::dream(
            &intelligence,
            &forgetful,
            &original,
            DreamMode::Reinterpretation,
            None,
            "test".to_owned(),
            4,
        );

        assert_eq!(dream.verdict(), DreamVerdict::Worse, "{dream:#?}");
        assert!(
            dream
                .findings()
                .iter()
                .any(|finding| finding.contains("left missing"))
        );

        // Nothing of the dream became real knowledge.
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
                EpisodeConfiguration::new(seed, true, true).with_activity(Activity::Reflection),
                false,
            )
        };

        let report = reflect(&mut memory, 1);

        assert_eq!(
            memory.knowledge().hypothesis(&rule).unwrap().status(),
            &HypothesisStatus::Refuted,
            "{report:#?}"
        );
        assert_eq!(memory.corrections().records().len(), 1);

        // The counterexample turns out to be unfounded: the examination
        // reconsiders the refutation it made.
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
    fn ablation_dreams_correct_misattributed_credit_reversibly() {
        let intelligence = intelligence(true, true);
        let mut memory = PersistentMemory::default();

        // The gatherer and the lister are first used alone, which records
        // what they acquire.
        intelligence.reason(
            &problem("without-oracle"),
            &mut memory,
            EpisodeConfiguration::new(1, true, true)
                .without_modules(BTreeSet::from(["oracle".to_owned()])),
            false,
        );

        // Then the oracle alone answers, and is credited for it.
        intelligence.reason(
            &problem("oracle-only"),
            &mut memory,
            EpisodeConfiguration::new(2, true, true)
                .without_modules(BTreeSet::from(["gatherer".to_owned(), "lister".to_owned()])),
            false,
        );

        let original = memory.experiences().episodes().last().unwrap().clone();

        assert!(
            original
                .invocations()
                .iter()
                .any(|invocation| invocation.module() == "oracle"
                    && invocation.outcome().is_success(true))
        );

        let dream = Dreaming::dream(
            &intelligence,
            &memory,
            &original,
            DreamMode::Historical,
            Some(DreamVariant::Ablation {
                module: "oracle".to_owned(),
            }),
            "is the oracle needed?".to_owned(),
            99,
        );

        assert!(
            matches!(
                dream.verdict(),
                DreamVerdict::Equivalent | DreamVerdict::Cheaper
            ),
            "{dream:#?}"
        );

        Dreaming::integrate(&mut memory, dream);

        intelligence.reason(
            &Event::internal(
                "reflection".to_owned(),
                vec!["learning_concern(?k, ?s)".parse().unwrap()],
            ),
            &mut memory,
            EpisodeConfiguration::new(100, true, true).with_activity(Activity::Reflection),
            false,
        );

        let correction = memory
            .corrections()
            .records()
            .iter()
            .find(|record| {
                matches!(record.correction(), Correction::ReassessInvocation { module, .. } if module == "oracle")
            })
            .expect("no reassessment of the oracle")
            .id();

        assert!(!memory.corrections().reassessments().is_empty());

        crate::engine::memory_correction::MemoryCorrection::revert(
            &mut memory,
            correction,
            "test".to_owned(),
        )
        .unwrap();

        assert!(memory.corrections().reassessments().is_empty());
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

        // Nothing is ever worth more than certainty: it abstains.
        assert!(
            Autonomy::new(&intelligence, &parameters, true, true, 1.01)
                .run(&mut memory, 5, 1)
                .is_empty()
        );

        let records =
            Autonomy::new(&intelligence, &parameters, true, true, 0.0).run(&mut memory, 6, 2);

        assert_eq!(records.len(), 6);
        assert!(
            records
                .iter()
                .any(|record| record.activity() == Activity::Dream)
        );
        assert_eq!(memory.activities().records().len(), 6);
        assert_eq!(
            memory
                .real_episodes()
                .filter(|episode| episode.activity() == Activity::Action)
                .count(),
            3
        );
    }
}
