use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

use chrono::Utc;
use uuid::Uuid;

use crate::{
    acquisition::{access_mode::AccessMode, external_access::ExternalAccess},
    capabilities::imagined_capability::ImaginedCapability,
    cognition::{
        capability_model::CapabilityModel, cognitive_module::CognitiveModule,
        module_context::ModuleContext, module_descriptor::ModuleDescriptor,
        module_environment::ModuleEnvironment, module_registry::ModuleRegistry, proposal::Proposal,
        reflections::Reflections,
    },
    consumables::resource_meter::ResourceMeter,
    engine::{
        belief_revision::BeliefRevision, candidate::Candidate, consolidation::Consolidation,
        disposition::Disposition, episode_report::EpisodeReport,
        impasse_diagnosis::ImpasseDiagnosis, learning_parameters::LearningParameters,
        memory_correction::MemoryCorrection, module_selector::ModuleSelector,
        perception::Perception, question_answering::QuestionAnswering,
    },
    events::event::Event,
    influences::{
        influence::Influence, influence_exposure::InfluenceExposure, influence_kind::InfluenceKind,
    },
    latent::occurrence::Occurrence,
    mathematics::beta_distribution::BetaDistribution,
    mathematics::random_source::RandomSource,
    memory::{
        acquisition_cache::AcquisitionCache, answer_status::AnswerStatus,
        episode_configuration::EpisodeConfiguration, episode_record::EpisodeRecord,
        invocation_outcome::InvocationOutcome, invocation_record::InvocationRecord,
        knowledge_exploitation::KnowledgeExploitation, knowledge_memory::KnowledgeMemory,
        persistent_memory::PersistentMemory, revision_record::RevisionRecord,
        routing_memory::RoutingMemory, termination::Termination,
    },
    planning::{plan::Plan, plan_record::PlanRecord},
    reasoning::{
        epistemic_kind::EpistemicKind, invalidation_cause::InvalidationCause, node_id::NodeId,
        producer::Producer, reasoning_graph::ReasoningGraph, working_memory::WorkingMemory,
    },
    representation::{
        bindings::Bindings, content::Content, provenance::Provenance, statement::Statement,
    },
    resources::initial_resources::InitialResources,
};

/// Central engine. It receives events, pursues questions, lets the
/// learned selector mobilise cognitive modules on a shared working memory,
/// revises its beliefs when contradictions appear, answers with
/// justifications and finally integrates what it learned.
///
/// Beyond reasoning, it keeps track of what holds now versus what was
/// remembered, of the plans proposed by planning modules and of the
/// external resources consumed, so that every decision weighs what remains
/// to gain against time and the scarcity of resources.
pub(crate) struct Intelligence {
    registry: ModuleRegistry,
    resources: InitialResources,
    parameters: LearningParameters,
}

/// Mutable state of one episode beside the working memory.
#[derive(Default)]
struct EpisodeTrace {
    revisions: Vec<RevisionRecord>,
    withdrawn: Vec<(NodeId, String)>,
    plans: Vec<PlanRecord>,
    exploitation: KnowledgeExploitation,
    reflections: Reflections,
}

/// What came out of integrating the proposals of one invocation.
#[derive(Default)]
struct Integration {
    produced: Vec<usize>,
    features: BTreeSet<String>,
    rejected: Vec<String>,
}

impl Intelligence {
    pub(crate) fn new(
        registry: ModuleRegistry,
        resources: InitialResources,
        parameters: LearningParameters,
    ) -> Self {
        Self {
            registry,
            resources,
            parameters,
        }
    }

    pub(crate) fn resources(&self) -> &InitialResources {
        &self.resources
    }

    pub(crate) fn descriptors(&self) -> impl Iterator<Item = &ModuleDescriptor> {
        self.registry
            .modules()
            .iter()
            .map(|module| module.descriptor())
    }

    /// Reasons without any internal state (laboratory condition).
    #[cfg(test)]
    pub(crate) fn reason(
        &self,
        event: &Event,
        memory: &mut PersistentMemory,
        configuration: EpisodeConfiguration,
        detailed: bool,
    ) -> EpisodeReport {
        self.reason_with(
            event,
            memory,
            configuration,
            detailed,
            &mut crate::engine::neutral_disposition::NeutralDisposition,
        )
    }

    /// Reasons under the modulation of an internal state, which every
    /// occurrence of the episode is reported to as it happens.
    pub(crate) fn reason_with(
        &self,
        event: &Event,
        memory: &mut PersistentMemory,
        configuration: EpisodeConfiguration,
        detailed: bool,
        disposition: &mut dyn Disposition,
    ) -> EpisodeReport {
        let episode = Uuid::now_v7();
        let started_at = Utc::now();
        let clock = Instant::now();
        let configuration = configuration
            .with_parameters(&self.parameters)
            .modulated(disposition.modulation());
        let mut random = RandomSource::new(configuration.seed());
        let mut working = WorkingMemory::new(event.questions().to_vec());
        let mut trace = EpisodeTrace::default();

        let empty_knowledge = KnowledgeMemory::default();
        let empty_routing = RoutingMemory::default();

        let knowledge = if configuration.acquired_knowledge() {
            memory.knowledge()
        } else {
            &empty_knowledge
        };

        let routing = if configuration.learned_routing() {
            memory.routing()
        } else {
            &empty_routing
        };

        // Replays (dreams) reach nothing outside: they only read what was
        // recorded, and start without the live state of the resources.
        let empty_cache = AcquisitionCache::default();
        let cache = if configuration.acquisition_cache() {
            memory.acquisitions()
        } else {
            &empty_cache
        };
        let live = configuration.access() == AccessMode::Live;
        let meter = ResourceMeter::new(
            if live {
                memory.resources().clone()
            } else {
                BTreeMap::new()
            },
            configuration.resource_budgets().clone(),
        );
        let access = match configuration.access() {
            AccessMode::Live => ExternalAccess::new(meter, cache),
            AccessMode::Replay { as_of, since } => {
                ExternalAccess::replay(meter, cache, as_of, since)
            }
        };

        // Capabilities imagined in a dream exist only for its episodes.
        let imagined: Vec<ImaginedCapability> = configuration
            .imagined_capabilities()
            .iter()
            .cloned()
            .map(ImaginedCapability::new)
            .collect();
        let modules: Vec<&dyn CognitiveModule> = self
            .registry
            .modules()
            .iter()
            .map(|module| module.as_ref())
            .chain(imagined.iter().map(|module| module as &dyn CognitiveModule))
            .collect();

        let mut initial_errors = Vec::new();

        for (content, kind, producer, provenance) in self.initial_contents(event, live) {
            match working.graph_mut().insert(
                content.clone(),
                kind,
                producer,
                Vec::new(),
                provenance,
                0,
            ) {
                Ok(insertion) => {
                    Self::revise(&mut working, insertion.contradictions(), 0, &mut trace);
                }
                Err(error) => initial_errors.push(format!("{content}: {error}")),
            }
        }

        // Setting of a dream scene, each element with how it came there.
        for element in configuration.imagined_elements() {
            let provenance = (element.kind() == EpistemicKind::Imagined)
                .then(|| Provenance::new("imagination", Some(element.origin().to_owned()), None));

            match working.graph_mut().insert(
                element.content().clone(),
                element.kind(),
                Producer::Imagination(element.origin().to_owned()),
                Vec::new(),
                provenance,
                0,
            ) {
                Ok(insertion) => {
                    Self::revise(&mut working, insertion.contradictions(), 0, &mut trace);
                }
                Err(error) => initial_errors.push(format!("{}: {error}", element.content())),
            }
        }

        // What introspection perceives of the individual itself: observations
        // of the self, each aspect a scope that newer perceptions supersede.
        let (self_observations, self_revision) = configuration.self_observations();

        for (statement, scope) in self_observations {
            match working.graph_mut().insert(
                Content::Statement(statement.clone()),
                EpistemicKind::Observation,
                Producer::Perception,
                Vec::new(),
                Some(Provenance::new(
                    "introspection",
                    Some(scope.clone()),
                    self_revision.map(str::to_owned),
                )),
                0,
            ) {
                Ok(insertion) => {
                    Self::revise(&mut working, insertion.contradictions(), 0, &mut trace);
                }
                Err(error) => initial_errors.push(format!("{statement}: {error}")),
            }
        }

        // Influences of the unconscious bearing on this situation: their
        // contents enter as defeasible elements, their preferences weigh on
        // the choices; nothing is imposed.
        let situation: BTreeSet<String> = working
            .graph()
            .valid_nodes()
            .filter(|node| !matches!(node.producer(), Producer::Resource(_)))
            .flat_map(|node| Self::predicates(node.content()))
            .collect();
        let applicable: Vec<Influence> = if configuration.influenced() {
            memory
                .influences()
                .active()
                .filter(|influence| {
                    influence.applies_to(&situation)
                        && !configuration.excludes_influence(influence.id())
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        };
        let influence_evidence = memory.influences().evidence();
        let mut injected: BTreeMap<Uuid, Vec<NodeId>> = BTreeMap::new();
        let mut preferences: BTreeMap<String, Vec<(Uuid, bool, BetaDistribution)>> =
            BTreeMap::new();

        for influence in &applicable {
            let id = influence.id();
            let content = match influence.kind() {
                InfluenceKind::Conjecture { rule } => {
                    Some((Content::Rule(rule.clone()), EpistemicKind::Hypothesis))
                }
                InfluenceKind::Belief { statement } => Some((
                    Content::Statement(statement.clone()),
                    EpistemicKind::Hypothesis,
                )),
                InfluenceKind::PotentialNeed { pattern }
                | InfluenceKind::Doubt { statement: pattern } => {
                    Some((Content::Need(pattern.clone()), EpistemicKind::Need))
                }
                InfluenceKind::Favour { .. } | InfluenceKind::Question { .. } => None,
            };

            if let Some((content, kind)) = content {
                match working.graph_mut().insert(
                    content.clone(),
                    kind,
                    Producer::Unconscious(id.to_string()),
                    Vec::new(),
                    None,
                    0,
                ) {
                    Ok(insertion) => {
                        injected.entry(id).or_default().push(insertion.id());
                        Self::revise(&mut working, insertion.contradictions(), 0, &mut trace);
                    }
                    Err(error) => initial_errors.push(format!("{content}: {error}")),
                }
            }

            if let Some((module, favour)) = influence.kind().preference() {
                let (successes, failures) = influence_evidence
                    .get(&id)
                    .map(|evidence| evidence.outcomes())
                    .unwrap_or((0, 0));

                preferences.entry(module.to_owned()).or_default().push((
                    id,
                    favour,
                    BetaDistribution::posterior(
                        self.parameters.prior_successes(),
                        self.parameters.prior_failures(),
                        successes,
                        failures,
                    ),
                ));
            }
        }

        let remembered = self.remembered(knowledge);
        let capabilities = CapabilityModel::new(
            routing,
            &self.parameters,
            modules
                .iter()
                .map(|module| module.descriptor().clone())
                .collect(),
            remembered.clone(),
        );
        let environment = ModuleEnvironment::new(
            knowledge,
            &self.resources,
            &access,
            &capabilities,
            &self.parameters,
            memory,
        );

        let selector = ModuleSelector::new(routing, &self.parameters);
        let mut seen = vec![0_u64; modules.len()];
        let mut invocations: Vec<InvocationRecord> = Vec::new();
        let mut previous: Option<String> = None;

        let termination = loop {
            let step = invocations.len() + 1;

            if Self::questions_answered(&working) {
                break Termination::QuestionsAnswered;
            }

            let modulation = disposition.modulation();
            let budget = configuration
                .maximum_invocations()
                .unwrap_or(self.parameters.maximum_invocations());
            let budget = if modulation.invocation_factor() == 1.0 {
                budget
            } else {
                ((budget as f64 * modulation.invocation_factor()).round() as usize).max(1)
            };

            if invocations.len() >= budget {
                break Termination::InvocationBudgetExhausted;
            }

            // The reasoner may give up at any step, as its state disposes it.
            if !invocations.is_empty()
                && modulation.abandonment() > 0.0
                && random.uniform() < modulation.abandonment()
            {
                break Termination::Abandoned;
            }

            if clock.elapsed() >= self.parameters.maximum_duration() {
                break Termination::TimeBudgetExhausted;
            }

            let active_plan = trace.plans.iter().rev().find(|record| record.is_active());
            let mut candidates = self.candidates(
                &modules,
                &working,
                &seen,
                previous.as_deref(),
                &remembered,
                active_plan,
            );

            candidates.retain(|candidate| !configuration.excludes(candidate.name()));

            let Some((position, selection)) = selector.select(
                &candidates,
                access.meter(),
                &mut random,
                &preferences,
                &modulation,
            ) else {
                break Termination::Quiescent;
            };

            if selection.best_expected_utility() == f64::NEG_INFINITY {
                break Termination::ResourcesExhausted;
            }

            if selection.best_expected_utility() < self.parameters.minimum_utility() {
                break Termination::NothingWorthTrying;
            }

            let index = candidates[position].module();
            let module = modules[index];
            let descriptor = module.descriptor();
            let revision_before = working.graph().revision();
            let usage_before = access.meter().usage();
            let available_before = access.meter().availabilities();

            let revisions_before = trace.revisions.len();
            let started = Instant::now();
            let result = module.apply(&ModuleContext::new(
                working.graph(),
                &environment,
                seen[index],
                active_plan.map(PlanRecord::plan),
            ));
            let elapsed = started.elapsed();
            let duration = u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX);
            let usage = access.meter().usage().since(&usage_before);
            let actual_cost = self.parameters.cost_weight_per_second() * elapsed.as_secs_f64()
                + self.parameters.scarcity_weight()
                    * ResourceMeter::scarcity_cost_given(&available_before, usage.consumption());

            seen[index] = revision_before;

            let (integration, notes, plan, error) = match result {
                Ok(mut output) => {
                    let reflections = output.take_reflections();
                    let (proposals, mut notes, plan) = output.into_parts();
                    let mut integration = Self::integrate(
                        &mut working,
                        &mut trace,
                        descriptor.name(),
                        |kind| descriptor.produces(kind),
                        proposals,
                        step,
                        !live,
                    );

                    if descriptor.reflects() {
                        notes.extend(reflections.corrections().iter().map(|(correction, _, _)| {
                            format!("proposed correction: {correction}")
                        }));
                        trace.reflections.extend(reflections);
                    } else if !reflections.is_empty() {
                        integration.rejected.push(format!(
                            "corrections, experiments or influences proposed by {}, which is not reflective",
                            descriptor.name()
                        ));
                    }

                    (integration, notes, plan, None)
                }
                Err(error) => (Integration::default(), Vec::new(), None, Some(error)),
            };

            let succeeded = error.is_none();
            let mut record = InvocationRecord::new(
                step,
                descriptor.name().to_owned(),
                selection,
                duration,
                integration.produced,
                integration.features.into_iter().collect(),
                integration.rejected,
                notes,
                error,
            );

            if let Some(active) = trace
                .plans
                .iter_mut()
                .rev()
                .find(|record| record.is_active())
            {
                active.account(
                    descriptor.name(),
                    step,
                    succeeded,
                    elapsed.as_secs_f64(),
                    &usage,
                    actual_cost,
                );
            }

            match plan {
                Some(plan) if descriptor.plans() => {
                    Self::adopt(&mut trace, plan, step);
                    record.adopt_plan();
                }
                Some(_) => record.reject(format!(
                    "plan proposed by {}, which does not declare planning",
                    descriptor.name()
                )),
                None => {}
            }

            Self::feel_invocation(
                disposition,
                &record,
                &usage,
                trace.revisions.len() - revisions_before,
            );
            record.account(usage, actual_cost);
            invocations.push(record);
            previous = Some(descriptor.name().to_owned());
        };

        disposition.feel(Occurrence::plain(format!(
            "termination:{}",
            termination.label()
        )));

        let mut notes_from_corrections = Vec::new();
        let graph = working.graph();
        let mut answers = Vec::new();
        let mut support = BTreeSet::new();

        for question in working.questions() {
            let (answer, answer_support) = QuestionAnswering::answer(graph, question);

            answers.push(answer);
            support.extend(answer_support);
        }

        trace.exploitation.supported_answers(
            support
                .iter()
                .filter_map(|id| graph.node(*id))
                .filter(|node| node.kind() == EpistemicKind::Recollection)
                .count(),
        );

        let last_step = invocations.len();
        let mut achieved_plans = BTreeSet::new();

        for record in &mut trace.plans {
            let achieved = record.plan().goals().iter().all(|goal| graph.answers(goal));

            record.conclude(last_step, achieved);

            if record.achieved() {
                achieved_plans.insert(record.adopted_at());
            }
        }

        Self::assess(graph, &support, &achieved_plans, &mut invocations);

        for answer in &answers {
            disposition.feel(Occurrence::plain(format!(
                "answer:{}",
                answer.status().label()
            )));
        }

        let definite_answers = answers
            .iter()
            .filter(|answer| {
                matches!(
                    answer.status(),
                    AnswerStatus::Proven | AnswerStatus::Refuted
                )
            })
            .count();
        let exposures: Vec<InfluenceExposure> = applicable
            .iter()
            .map(|influence| {
                let id = influence.id();
                let nodes = injected.get(&id).cloned().unwrap_or_default();
                let decisive = invocations
                    .iter()
                    .filter(|invocation| invocation.selection().influences().contains(&id))
                    .map(|invocation| (invocation.step(), invocation.outcome()))
                    .collect();

                InfluenceExposure::new(
                    id,
                    episode,
                    configuration.activity(),
                    nodes.len(),
                    decisive,
                    nodes.iter().any(|node| support.contains(node)),
                    nodes
                        .iter()
                        .any(|node| graph.node(*node).is_some_and(|node| !node.is_valid())),
                    definite_answers,
                )
            })
            .collect();

        let descriptors: Vec<&ModuleDescriptor> =
            modules.iter().map(|module| module.descriptor()).collect();
        let impasses = ImpasseDiagnosis::diagnose(graph, &descriptors, &invocations);
        let resource_states = access.meter().states();
        let total_usage = access.meter().usage();
        let (_, store) = access.into_parts();
        let (stored, used) = store.into_parts();

        if live {
            *memory.resources_mut() = resource_states.clone();

            if configuration.acquisition_cache() {
                memory.acquisitions_mut().integrate(stored, used);
            }
        }

        // What the examination of the cognition concluded: corrections are
        // applied (journaled and reversible), experiments queued.
        let (proposed_corrections, requested, proposed, examined) = trace.reflections.into_parts();
        let mut corrections = Vec::new();

        for dream in examined {
            memory.dreams_mut().examine(dream);
        }

        for (correction, basis, justification) in proposed_corrections {
            let description = correction.to_string();

            match MemoryCorrection::apply(memory, episode, correction, basis, justification) {
                Ok(id) => corrections.push(format!("{id}: {description}")),
                Err(error) => notes_from_corrections.push(error),
            }
        }

        let applied: Vec<String> = applicable
            .iter()
            .map(|influence| format!("{}: {}", influence.id(), influence.kind()))
            .collect();

        for exposure in exposures {
            memory.influences_mut().expose(exposure);
        }

        let mut proposed_influences = Vec::new();

        for influence in proposed {
            let description = format!("{}", influence.kind());

            if memory.influences_mut().propose(influence) {
                proposed_influences.push(description);
            }
        }

        let mut experiments = Vec::new();

        for request in requested {
            if memory.dreams_mut().request(request.clone()) {
                experiments.push(request);
            }
        }

        let (knowledge_memory, routing_memory, experiences) = memory.parts_mut();

        let changes = Consolidation::integrate(graph, knowledge_memory, episode, &trace.withdrawn);

        let mut notes = initial_errors;
        notes.append(&mut notes_from_corrections);
        notes.sort();

        let record = EpisodeRecord::new(
            episode,
            event.id().to_owned(),
            started_at,
            u64::try_from(clock.elapsed().as_micros()).unwrap_or(u64::MAX),
            configuration,
            working.questions().to_vec(),
            invocations,
            trace.revisions,
            answers,
            changes.hypotheses_proposed().to_vec(),
            changes.hypotheses_refuted().to_vec(),
            impasses,
            termination,
            notes,
        )
        .with_execution(
            trace.plans,
            total_usage,
            resource_states,
            trace.exploitation,
        )
        .with_reflection(corrections, experiments)
        .with_influences(applied, proposed_influences)
        .with_event(event.clone());

        routing_memory.learn(&record);
        experiences.record(record.clone());

        EpisodeReport::new(
            record,
            changes,
            routing_memory.episodes_learned(),
            detailed.then_some(working),
        )
    }

    /// Reports what one invocation did: the action, what it produced or
    /// failed to, what it consumed or was refused, the revisions it caused
    /// and the influences that decided it.
    fn feel_invocation(
        disposition: &mut dyn Disposition,
        record: &InvocationRecord,
        usage: &crate::consumables::resource_usage::ResourceUsage,
        revisions: usize,
    ) {
        let module = record.module();

        disposition.feel(Occurrence::plain(format!("invoke:{module}")));

        if record.error().is_some() {
            disposition.feel(Occurrence::plain(format!("error:{module}")));
        } else if record.produced().is_empty() {
            disposition.feel(Occurrence::plain(format!("barren:{module}")));
        } else {
            disposition.feel(Occurrence::new(
                format!("produce:{module}"),
                (1.0 + record.produced().len() as f64).ln(),
            ));
        }

        for (resource, units) in usage.consumption() {
            disposition.feel(Occurrence::new(
                format!("consume:{resource}"),
                (1.0 + units.max(0.0)).ln(),
            ));
        }

        if usage.refused() > 0 {
            disposition.feel(Occurrence::new(
                "refused:resource",
                (1.0 + usage.refused() as f64).ln(),
            ));
        }

        if !record.selection().influences().is_empty() {
            disposition.feel(Occurrence::plain("influenced:decision"));
        }

        if revisions > 0 {
            disposition.feel(Occurrence::new(
                "revision:belief",
                (1.0 + revisions as f64).ln(),
            ));
        }
    }

    fn predicates(content: &Content) -> Vec<String> {
        match content {
            Content::Statement(statement) | Content::Need(statement) => {
                vec![statement.predicate().to_owned()]
            }
            Content::Rule(rule) => rule
                .premises()
                .iter()
                .chain([rule.conclusion()])
                .map(|statement| statement.predicate().to_owned())
                .collect(),
            Content::Document(_) => Vec::new(),
        }
    }

    /// Kinds of information the long-term memory can provide, with how
    /// outdated they are on average.
    fn remembered(&self, knowledge: &KnowledgeMemory) -> BTreeMap<String, f64> {
        let now = Utc::now();
        let prior = self.parameters.change_prior();
        let mut totals: BTreeMap<String, (f64, usize)> = BTreeMap::new();

        for entry in knowledge.active_entries() {
            let (_, staleness) = knowledge.currency(entry, now, prior);
            let total = totals.entry(entry.statement().signature()).or_default();

            total.0 += staleness;
            total.1 += 1;
        }

        for record in knowledge.hypotheses() {
            totals
                .entry(record.rule().conclusion().signature())
                .or_default()
                .1 += 1;
        }

        totals
            .into_iter()
            .map(|(signature, (staleness, count))| (signature, staleness / count.max(1) as f64))
            .collect()
    }

    /// Follows a newly proposed plan, unless it repeats the active one;
    /// the active plan is then recorded as revised.
    fn adopt(trace: &mut EpisodeTrace, plan: Plan, step: usize) {
        if let Some(active) = trace
            .plans
            .iter_mut()
            .rev()
            .find(|record| record.is_active())
        {
            if active.plan() == &plan {
                return;
            }

            let reason = if active.plan().goals() != plan.goals() {
                "the open goals changed".to_owned()
            } else if active.has_failures() {
                "a planned step failed".to_owned()
            } else {
                format!(
                    "new information favoured {} over {}",
                    plan.strategy(),
                    active.plan().strategy()
                )
            };

            active.revise(step, reason);
        }

        trace.plans.push(PlanRecord::new(plan, step));
    }

    /// Contents an episode starts from. What the event shows is an
    /// observation, unless the event is replayed: then it is a replay.
    fn initial_contents(
        &self,
        event: &Event,
        live: bool,
    ) -> Vec<(Content, EpistemicKind, Producer, Option<Provenance>)> {
        // What another individual transmitted is its testimony, never an
        // observation of the world.
        let perceived = if event.source().starts_with(Event::INDIVIDUAL_SOURCE) {
            EpistemicKind::Testimony
        } else if live {
            EpistemicKind::Observation
        } else {
            EpistemicKind::Replay
        };
        let resource = Producer::Resource(InitialResources::KNOWLEDGE_FILE.to_owned());
        let knowledge = self.resources.knowledge();
        let provenance = Provenance::new(
            format!("event:{}", event.source()),
            None,
            Some(event.id().to_owned()),
        );

        knowledge
            .statements()
            .iter()
            .cloned()
            .map(Content::Statement)
            .chain(knowledge.rules().iter().cloned().map(Content::Rule))
            .map(|content| (content, EpistemicKind::Premise, resource.clone(), None))
            .chain(Perception::perceive(event).into_iter().map(|content| {
                (
                    content,
                    perceived,
                    Producer::Perception,
                    Some(provenance.clone()),
                )
            }))
            .chain(event.questions().iter().map(|question| {
                (
                    Content::Need(question.clone()),
                    EpistemicKind::Need,
                    Producer::Perception,
                    None,
                )
            }))
            .collect()
    }

    /// Every question has a definite answer known to hold now.
    fn questions_answered(working: &WorkingMemory) -> bool {
        !working.questions().is_empty()
            && working
                .questions()
                .iter()
                .all(|question| question.is_ground() && working.graph().answers_currently(question))
    }

    /// Modules having valid, unprocessed content they consume, with the
    /// features describing that content and the stake of acting on it.
    #[allow(clippy::too_many_arguments)]
    fn candidates(
        &self,
        modules: &[&dyn CognitiveModule],
        working: &WorkingMemory,
        seen: &[u64],
        previous: Option<&str>,
        remembered: &BTreeMap<String, f64>,
        active_plan: Option<&PlanRecord>,
    ) -> Vec<Candidate> {
        let graph = working.graph();
        let mut candidates = Vec::new();
        let ready = active_plan.map(PlanRecord::ready_steps).unwrap_or_default();

        for (index, module) in modules.iter().enumerate() {
            let descriptor = module.descriptor();
            let mut features = BTreeSet::new();
            let mut stake: f64 = 0.0;

            for node in graph.valid_nodes() {
                if node.revision() <= seen[index] || !descriptor.consumes(node.content().kind()) {
                    continue;
                }

                if let Some(pattern) = node.content().need() {
                    if !descriptor.supplies(&pattern.signature())
                        || !ModuleContext::open_for(graph, pattern, descriptor)
                    {
                        continue;
                    }

                    // Remembered answers leave at stake only the chance
                    // that they are outdated.
                    match graph.answer_staleness(pattern) {
                        Some(staleness) => {
                            stake = stake.max(staleness);
                            features.insert("verification".to_owned());
                        }
                        None => stake = 1.0,
                    }

                    if remembered.contains_key(&pattern.signature()) {
                        features.insert(format!("remembered:{}", pattern.signature()));
                    }
                } else {
                    stake = 1.0;
                }

                features.insert(node.content().feature());
            }

            if features.is_empty() || stake <= 0.0 {
                continue;
            }

            features.extend(
                working
                    .questions()
                    .iter()
                    .map(|question| format!("question:{}", question.signature())),
            );

            if let Some(previous) = previous {
                features.insert(format!("after:{previous}"));
            }

            // Position of the module relative to the plan being followed:
            // whether following plans pays off is learned like the rest.
            match active_plan {
                Some(record) if descriptor.plans() => {
                    features.insert(if record.has_failures() {
                        "plan:failing".to_owned()
                    } else {
                        "plan:active".to_owned()
                    });
                }
                Some(record) => {
                    let steps = record.steps_of(descriptor.name());

                    features.insert(if steps.is_empty() {
                        "plan:outside".to_owned()
                    } else if steps.iter().any(|step| ready.contains(step)) {
                        "plan:ready".to_owned()
                    } else {
                        "plan:waiting".to_owned()
                    });
                }
                None if descriptor.plans() => {
                    features.insert("plan:none".to_owned());
                }
                None => {}
            }

            candidates.push(Candidate::new(
                index,
                descriptor.name().to_owned(),
                features.into_iter().collect(),
                stake,
            ));
        }

        candidates
    }

    /// Inserts the proposals of a module into the graph, enforcing its
    /// declared contract, then reconciles what is remembered with what was
    /// just observed and revises beliefs if contradictions appeared.
    ///
    /// Observations sharing a source, scope and revision describe that
    /// whole scope at that revision. A recollection from the same revision
    /// is therefore confirmed; one from another revision, not observed
    /// again, is superseded. A recollection of a scope already observed
    /// in another revision is refused.
    fn integrate(
        working: &mut WorkingMemory,
        trace: &mut EpisodeTrace,
        module: &str,
        allowed: impl Fn(EpistemicKind) -> bool,
        proposals: Vec<Proposal>,
        step: usize,
        replaying: bool,
    ) -> Integration {
        let mut integration = Integration::default();
        let current_scopes = Self::current_scopes(working.graph());
        let mut observed: BTreeMap<(String, String), (Option<String>, NodeId)> = BTreeMap::new();
        let mut observed_statements: BTreeSet<Statement> = BTreeSet::new();

        for proposal in proposals {
            if !allowed(proposal.kind()) {
                integration
                    .rejected
                    .push(format!("undeclared result kind {:?}", proposal.kind()));
                continue;
            }

            let mut staleness = proposal.staleness();

            if proposal.kind() == EpistemicKind::Recollection
                && let Some(provenance) = proposal.provenance()
                && let Some(scope) = provenance.scope()
                && let Some(revisions) =
                    current_scopes.get(&(provenance.source().to_owned(), scope.to_owned()))
            {
                if revisions.contains(&provenance.revision().map(str::to_owned)) {
                    staleness = 0.0;
                } else {
                    trace.exploitation.refused_outdated();
                    integration.rejected.push(format!(
                        "{}: superseded by the current observation of {scope}",
                        proposal.content()
                    ));
                    continue;
                }
            }

            let (content, kind, premises, provenance) = proposal.into_parts();
            // In a dream, what modules observe was only reproduced from
            // recorded data: it is never a new observation of the world.
            let kind = if replaying && kind == EpistemicKind::Observation {
                EpistemicKind::Replay
            } else {
                kind
            };
            let feature = content.feature();
            let description = content.to_string();
            let scope = provenance.as_ref().and_then(|provenance| {
                Some((
                    (
                        provenance.source().to_owned(),
                        provenance.scope()?.to_owned(),
                    ),
                    provenance.revision().map(str::to_owned),
                ))
            });
            let statement = content.statement().cloned();

            match working.graph_mut().insert_dated(
                content,
                kind,
                Producer::Module(module.to_owned()),
                premises,
                provenance,
                step,
                staleness,
            ) {
                Ok(insertion) => {
                    if insertion.created() {
                        integration.produced.push(insertion.id().index());
                        integration.features.insert(feature);

                        if kind == EpistemicKind::Recollection {
                            trace.exploitation.recalled(1);
                        }
                    }

                    if kind.is_observed() {
                        if let Some((scope, revision)) = scope {
                            observed.entry(scope).or_insert((revision, insertion.id()));
                        }

                        if let Some(statement) = statement {
                            observed_statements.insert(statement);
                        }
                    }

                    Self::revise(working, insertion.contradictions(), step, trace);
                }
                Err(error) => integration.rejected.push(format!("{description}: {error}")),
            }
        }

        if !observed.is_empty() {
            Self::reconcile(working, trace, &observed, &observed_statements, step);
        }

        integration
    }

    /// Source scopes observed during the episode, with their revisions.
    fn current_scopes(
        graph: &ReasoningGraph,
    ) -> BTreeMap<(String, String), BTreeSet<Option<String>>> {
        let mut scopes: BTreeMap<(String, String), BTreeSet<Option<String>>> = BTreeMap::new();

        for node in graph.valid_nodes() {
            if !node.kind().is_observed() {
                continue;
            }

            if let Some(provenance) = node.provenance()
                && let Some(scope) = provenance.scope()
            {
                scopes
                    .entry((provenance.source().to_owned(), scope.to_owned()))
                    .or_default()
                    .insert(provenance.revision().map(str::to_owned));
            }
        }

        scopes
    }

    fn reconcile(
        working: &mut WorkingMemory,
        trace: &mut EpisodeTrace,
        observed: &BTreeMap<(String, String), (Option<String>, NodeId)>,
        observed_statements: &BTreeSet<Statement>,
        step: usize,
    ) {
        let graph = working.graph();
        let mut confirmed = Vec::new();
        let mut superseded = Vec::new();

        for node in graph.valid_nodes() {
            if node.kind() != EpistemicKind::Recollection || node.is_current() {
                continue;
            }

            if node
                .content()
                .statement()
                .is_some_and(|statement| observed_statements.contains(statement))
            {
                confirmed.push(node.id());
                continue;
            }

            let Some(provenance) = node.provenance() else {
                continue;
            };

            let Some(scope) = provenance.scope() else {
                continue;
            };

            if let Some((revision, by)) =
                observed.get(&(provenance.source().to_owned(), scope.to_owned()))
            {
                if revision.as_deref() == provenance.revision() {
                    confirmed.push(node.id());
                } else {
                    superseded.push((node.id(), *by));
                }
            }
        }

        let graph = working.graph_mut();

        trace.exploitation.confirmed(graph.confirm(&confirmed));
        trace.exploitation.superseded(superseded.len());

        for (id, by) in superseded {
            graph.invalidate(id, InvalidationCause::Superseded { by }, step);
        }
    }

    fn revise(
        working: &mut WorkingMemory,
        contradictions: &[(NodeId, NodeId)],
        step: usize,
        trace: &mut EpisodeTrace,
    ) {
        for (positive, negative) in contradictions {
            let Some((record, withdrawn)) =
                BeliefRevision::resolve(working, *positive, *negative, step)
            else {
                continue;
            };

            let reason = format!(
                "withdrawn at step {step} to resolve the contradiction between {positive} and {negative}"
            );

            trace
                .withdrawn
                .extend(withdrawn.into_iter().map(|id| (id, reason.clone())));
            trace.revisions.push(record);
        }
    }

    /// Assigns each invocation a verifiable outcome once the episode is over.
    /// A need contributes when an answer-supporting statement fulfils it;
    /// satisfying a need, or formulating one later satisfied, is progress,
    /// and so is proposing a plan whose goals were reached.
    fn assess(
        graph: &ReasoningGraph,
        support: &BTreeSet<NodeId>,
        achieved_plans: &BTreeSet<usize>,
        invocations: &mut [InvocationRecord],
    ) {
        let supporting: Vec<&Statement> = support
            .iter()
            .filter_map(|id| graph.node(*id)?.content().statement())
            .collect();

        let contributes = |id: NodeId| {
            if support.contains(&id) {
                return true;
            }

            graph
                .node(id)
                .and_then(|node| node.content().need())
                .is_some_and(|pattern| {
                    supporting
                        .iter()
                        .any(|statement| Bindings::default().unify(pattern, statement).is_some())
                })
        };

        // Needs satisfied after they were formulated, and the statements
        // that satisfied them: both sides made verifiable progress.
        let mut progress = BTreeSet::new();

        for (need, by) in graph.satisfactions() {
            if by > need {
                progress.insert(*need);
                progress.insert(*by);
            }
        }

        for invocation in invocations {
            if invocation.error().is_some() {
                continue;
            }

            let produced: Vec<NodeId> = invocation
                .produced()
                .iter()
                .map(|index| NodeId::new(*index))
                .collect();

            let planned = invocation.adopted_plan() && achieved_plans.contains(&invocation.step());

            let outcome = if produced.iter().any(|id| contributes(*id)) {
                InvocationOutcome::Contributed
            } else if planned || produced.iter().any(|id| progress.contains(id)) {
                InvocationOutcome::Advanced
            } else if produced.is_empty() {
                InvocationOutcome::Unproductive
            } else if produced
                .iter()
                .any(|id| graph.node(*id).is_some_and(|node| node.is_valid()))
            {
                InvocationOutcome::Retained
            } else {
                InvocationOutcome::Invalidated
            };

            invocation.assess(outcome);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Intelligence;
    use crate::{
        capabilities::module_catalog::ModuleCatalog,
        engine::learning_parameters::LearningParameters,
        events::event::Event,
        memory::{
            answer_status::AnswerStatus, episode_configuration::EpisodeConfiguration,
            hypothesis_status::HypothesisStatus, persistent_memory::PersistentMemory,
        },
        resources::initial_resources::InitialResources,
    };

    fn event(id: &str, statements: &[&str], rules: &[&str], questions: &[&str]) -> Event {
        let list = |items: &[&str]| {
            items
                .iter()
                .map(|item| format!("{item:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        };

        ron::de::from_str(&format!(
            r#"(id: "{id}", source: "test", occurred_at: "2026-11-26T00:00:00Z",
                payload: assertions(statements: [{}], rules: [{}]), questions: [{}])"#,
            list(statements),
            list(rules),
            list(questions),
        ))
        .unwrap()
    }

    fn intelligence() -> Intelligence {
        Intelligence::new(
            ModuleCatalog::standard(true),
            InitialResources::default(),
            LearningParameters::default(),
        )
    }

    #[test]
    fn induces_uses_and_then_refutes_a_hypothesis() {
        let mut memory = PersistentMemory::default();

        let first = intelligence().reason(
            &event(
                "first",
                &[
                    "human(socrates)",
                    "human(plato)",
                    "human(aristotle)",
                    "human(hercules)",
                    "mortal(socrates)",
                    "mortal(plato)",
                    "mortal(aristotle)",
                    "release(a, v1.2.0)",
                    "release(b, v1.10.0)",
                ],
                &["older(?x, ?y) :- release(?x, ?u), release(?y, ?v), less_than(?u, ?v)"],
                &["mortal(hercules)", "older(a, b)", "older(b, a)"],
            ),
            &mut memory,
            EpisodeConfiguration::new(1, true, true),
            false,
        );

        let statuses: Vec<_> = first
            .episode_answers()
            .iter()
            .map(|answer| answer.status())
            .collect();

        assert_eq!(
            statuses,
            vec![
                AnswerStatus::Proven,
                AnswerStatus::Proven,
                AnswerStatus::Undetermined
            ],
            "{first:#?}"
        );
        assert!(
            memory
                .knowledge()
                .hypotheses()
                .any(|record| record.rule().to_string() == "mortal(?a0) :- human(?a0)")
        );

        let second = intelligence().reason(
            &event(
                "second",
                &["human(zeus)", "not mortal(zeus)"],
                &[],
                &["mortal(?who)"],
            ),
            &mut memory,
            EpisodeConfiguration::new(2, true, true),
            false,
        );

        assert!(!second.episode_revisions().is_empty(), "{second:#?}");

        let hypothesis = memory
            .knowledge()
            .hypotheses()
            .find(|record| record.rule().to_string() == "mortal(?a0) :- human(?a0)")
            .unwrap();

        assert_eq!(hypothesis.status(), &HypothesisStatus::Refuted);
        assert_eq!(memory.experiences().episodes().len(), 2);
    }

    #[test]
    fn resumes_a_need_once_its_prerequisite_is_satisfied() {
        let mut registry = crate::cognition::module_registry::ModuleRegistry::default();

        registry.register(crate::cognition::scripted_module::ScriptedModule::new(
            "gatherer",
            Some("item(?x)"),
            "fact(?x, 1)",
        ));
        registry.register(crate::cognition::scripted_module::ScriptedModule::new(
            "lister", None, "item(a)",
        ));

        let intelligence = Intelligence::new(
            registry,
            InitialResources::default(),
            LearningParameters::default(),
        );
        let mut memory = PersistentMemory::default();

        let report = intelligence.reason(
            &event("staged", &["context(x)"], &[], &["fact(?which, 1)"]),
            &mut memory,
            EpisodeConfiguration::new(4, true, true),
            false,
        );

        assert_eq!(
            report.episode_answers()[0].status(),
            AnswerStatus::Proven,
            "{report:#?}"
        );
    }

    fn remembering(
        statement: &str,
        source: &str,
        scope: &str,
        revision: &str,
        days: i64,
    ) -> PersistentMemory {
        let mut memory = PersistentMemory::default();
        let observed_at = chrono::Utc::now() - chrono::Duration::days(days);
        let (knowledge, _, _) = memory.parts_mut();

        knowledge.record_observation(
            statement.parse().unwrap(),
            Some(
                crate::representation::provenance::Provenance::new(
                    source,
                    Some(scope.to_owned()),
                    Some(revision.to_owned()),
                )
                .confirmed_at(observed_at),
            ),
            uuid::Uuid::now_v7(),
        );
        knowledge.verify_scope(source, scope, Some(revision), observed_at);
        memory
    }

    fn registry(
        modules: Vec<crate::cognition::scripted_module::ScriptedModule>,
    ) -> crate::cognition::module_registry::ModuleRegistry {
        let mut registry = crate::cognition::module_registry::ModuleRegistry::default();

        registry.register(crate::capabilities::knowledge_retrieval::KnowledgeRetrieval::new());

        for module in modules {
            registry.register(module);
        }

        registry
    }

    #[test]
    fn distinguishes_remembered_from_verified_answers() {
        let question = &["level(tank, ?level)"];

        // Without any way to observe the present, the remembered answer is
        // given as historical, with the probability that it is outdated.
        let mut memory = remembering("level(tank, 3)", "probe", "tank", "r1", 10);
        let report = Intelligence::new(
            registry(Vec::new()),
            InitialResources::default(),
            LearningParameters::default(),
        )
        .reason(
            &event("offline", &["context(x)"], &[], question),
            &mut memory,
            EpisodeConfiguration::new(1, true, true),
            false,
        );
        let answer = &report.episode_answers()[0];

        assert_eq!(answer.status(), AnswerStatus::Proven, "{report:#?}");
        assert!(!answer.is_current());
        assert!(answer.staleness() > 0.9);

        // An observation of the same revision confirms the recollection.
        let mut memory = remembering("level(tank, 3)", "probe", "tank", "r1", 10);
        let report = Intelligence::new(
            registry(vec![
                crate::cognition::scripted_module::ScriptedModule::new(
                    "probe",
                    None,
                    "level(tank, 3)",
                )
                .observing("probe", "tank", "r1"),
            ]),
            InitialResources::default(),
            LearningParameters::default().with_cost_weight_per_second(0.0),
        )
        .reason(
            &event("verified", &["context(x)"], &[], question),
            &mut memory,
            EpisodeConfiguration::new(2, true, true),
            false,
        );

        assert!(report.episode_answers()[0].is_current(), "{report:#?}");

        // A newer revision supersedes what was remembered from the old one.
        let mut memory = remembering("level(tank, 3)", "probe", "tank", "r1", 10);
        let report = Intelligence::new(
            registry(vec![
                crate::cognition::scripted_module::ScriptedModule::new(
                    "probe",
                    None,
                    "level(tank, 5)",
                )
                .observing("probe", "tank", "r2"),
            ]),
            InitialResources::default(),
            LearningParameters::default().with_cost_weight_per_second(0.0),
        )
        .reason(
            &event("changed", &["context(x)"], &[], question),
            &mut memory,
            EpisodeConfiguration::new(3, true, true),
            false,
        );
        let answer = &report.episode_answers()[0];

        assert_eq!(
            answer.answers(),
            &["level(tank, 5)"
                .parse::<crate::representation::statement::Statement>()
                .unwrap()],
            "{report:#?}"
        );
        assert!(answer.is_current());
        assert_eq!(
            memory
                .knowledge()
                .active_entries()
                .map(|entry| entry.statement().to_string())
                .filter(|statement| statement.starts_with("level"))
                .collect::<Vec<_>>(),
            vec!["level(tank, 5)".to_owned()]
        );
    }

    #[test]
    fn respects_exhausted_resources_without_forbidding_affordable_ones() {
        let intelligence = Intelligence::new(
            registry(vec![
                crate::cognition::scripted_module::ScriptedModule::new(
                    "meter_reader",
                    None,
                    "reading(1)",
                )
                .consuming("api", 1.0),
            ]),
            InitialResources::default(),
            LearningParameters::default(),
        );
        let mut memory = PersistentMemory::default();
        let ask = |name: &str| event(name, &["context(x)"], &[], &["reading(?value)"]);

        // The module is used while the resource is affordable, which
        // teaches what it consumes.
        let first = intelligence.reason(
            &ask("first"),
            &mut memory,
            EpisodeConfiguration::new(1, true, true),
            false,
        );

        assert_eq!(first.episode_answers()[0].status(), AnswerStatus::Proven);

        // An imposed budget of zero makes the acquisition impossible.
        let starved = intelligence.reason(
            &ask("starved"),
            &mut memory,
            EpisodeConfiguration::new(2, false, true)
                .with_resource_budgets([("api".to_owned(), 0.0)].into()),
            false,
        );

        assert_eq!(
            starved.episode().termination(),
            crate::memory::termination::Termination::ResourcesExhausted,
            "{starved:#?}"
        );
        assert_eq!(starved.episode().usage().refused(), 0);

        // A budget of a few units leaves it possible.
        let affordable = intelligence.reason(
            &ask("affordable"),
            &mut memory,
            EpisodeConfiguration::new(3, false, true)
                .with_resource_budgets([("api".to_owned(), 3.0)].into()),
            false,
        );

        assert_eq!(
            affordable.episode_answers()[0].status(),
            AnswerStatus::Proven
        );
    }

    #[test]
    fn plans_learned_prerequisites_and_learns_from_plans() {
        let mut registry = crate::cognition::module_registry::ModuleRegistry::default();

        registry.register(crate::capabilities::strategic_planning::StrategicPlanning::new());
        registry.register(crate::cognition::scripted_module::ScriptedModule::new(
            "gatherer",
            Some("item(?x)"),
            "fact(?x, 1)",
        ));
        registry.register(crate::cognition::scripted_module::ScriptedModule::new(
            "lister", None, "item(a)",
        ));

        let intelligence = Intelligence::new(
            registry,
            InitialResources::default(),
            LearningParameters::default(),
        );
        let mut memory = PersistentMemory::default();
        let mut adopted = Vec::new();

        for seed in 1..=12 {
            let report = intelligence.reason(
                &event(
                    &format!("e{seed}"),
                    &["context(x)"],
                    &[],
                    &["fact(?which, 1)"],
                ),
                &mut memory,
                EpisodeConfiguration::new(seed, false, true),
                false,
            );

            assert_eq!(report.episode_answers()[0].status(), AnswerStatus::Proven);
            adopted.extend(report.episode().plans().iter().cloned());
        }

        // Once the gatherer has been seen to need items, plans anticipate
        // that need and order the lister first.
        let anticipated = adopted
            .iter()
            .find(|record| record.plan().strategy().contains("item/1<-lister"))
            .unwrap_or_else(|| panic!("no plan anticipated the prerequisite: {adopted:#?}"));
        let steps = anticipated.plan().steps();

        assert_eq!(steps[0].module(), "lister");
        assert_eq!(steps[1].module(), "gatherer");
        assert_eq!(steps[1].depends_on(), &[0]);
        assert!(!memory.routing().strategies().is_empty());
    }
}
