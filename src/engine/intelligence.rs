use std::{collections::BTreeSet, time::Instant};

use chrono::Utc;
use uuid::Uuid;

use crate::{
    cognition::{
        module_context::ModuleContext, module_output::ModuleOutput, module_registry::ModuleRegistry,
    },
    engine::{
        belief_revision::BeliefRevision, candidate::Candidate, consolidation::Consolidation,
        episode_report::EpisodeReport, learning_parameters::LearningParameters,
        module_selector::ModuleSelector, perception::Perception,
        question_answering::QuestionAnswering,
    },
    events::event::Event,
    mathematics::random_source::RandomSource,
    memory::{
        episode_configuration::EpisodeConfiguration, episode_record::EpisodeRecord,
        invocation_outcome::InvocationOutcome, invocation_record::InvocationRecord,
        knowledge_memory::KnowledgeMemory, persistent_memory::PersistentMemory,
        revision_record::RevisionRecord, routing_memory::RoutingMemory, termination::Termination,
    },
    reasoning::{
        epistemic_kind::EpistemicKind, node_id::NodeId, producer::Producer,
        reasoning_graph::ReasoningGraph, working_memory::WorkingMemory,
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

    pub(crate) fn reason(
        &self,
        event: &Event,
        memory: &mut PersistentMemory,
        configuration: EpisodeConfiguration,
        detailed: bool,
    ) -> EpisodeReport {
        let episode = Uuid::now_v7();
        let started_at = Utc::now();
        let clock = Instant::now();
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

        let mut initial_errors = Vec::new();

        for (content, kind, producer, provenance) in self.initial_contents(event) {
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

        let selector = ModuleSelector::new(routing, &self.parameters);
        let mut seen = vec![0_u64; self.registry.modules().len()];
        let mut invocations: Vec<InvocationRecord> = Vec::new();
        let mut previous: Option<String> = None;

        let termination = loop {
            let step = invocations.len() + 1;

            if Self::questions_answered(&working) {
                break Termination::QuestionsAnswered;
            }

            if invocations.len() >= self.parameters.maximum_invocations() {
                break Termination::InvocationBudgetExhausted;
            }

            if clock.elapsed() >= self.parameters.maximum_duration() {
                break Termination::TimeBudgetExhausted;
            }

            let candidates = self.candidates(&working, &seen, previous.as_deref());

            let Some((position, selection)) = selector.select(&candidates, &mut random) else {
                break Termination::Quiescent;
            };

            if selection.utility() < self.parameters.minimum_utility() {
                break Termination::NothingWorthTrying;
            }

            let index = candidates[position].module();
            let module = &self.registry.modules()[index];
            let descriptor = module.descriptor();
            let revision_before = working.graph().revision();

            let started = Instant::now();
            let result = module.apply(&ModuleContext::new(
                working.graph(),
                knowledge,
                &self.resources,
                seen[index],
            ));
            let duration = u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX);

            seen[index] = revision_before;

            let (integration, notes, error) = match result {
                Ok(output) => {
                    let (integration, notes) = Self::integrate(
                        &mut working,
                        &mut trace,
                        descriptor.name(),
                        |kind| descriptor.produces(kind),
                        output,
                        step,
                    );

                    (integration, notes, None)
                }
                Err(error) => (Integration::default(), Vec::new(), Some(error)),
            };

            invocations.push(InvocationRecord::new(
                step,
                descriptor.name().to_owned(),
                selection,
                duration,
                integration.produced,
                integration.features.into_iter().collect(),
                integration.rejected,
                notes,
                error,
            ));

            previous = Some(descriptor.name().to_owned());
        };

        let graph = working.graph();
        let mut answers = Vec::new();
        let mut support = BTreeSet::new();

        for question in working.questions() {
            let (answer, answer_support) = QuestionAnswering::answer(graph, question);

            answers.push(answer);
            support.extend(answer_support);
        }

        Self::assess(graph, &support, &mut invocations);

        let open_needs: Vec<Statement> = graph
            .valid_nodes()
            .filter_map(|node| node.content().need())
            .filter(|pattern| !graph.answers(pattern))
            .cloned()
            .collect();

        let (knowledge_memory, routing_memory, experiences) = memory.parts_mut();

        let changes = Consolidation::integrate(graph, knowledge_memory, episode, &trace.withdrawn);

        let mut notes = initial_errors;
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
            open_needs,
            termination,
            notes,
        );

        routing_memory.learn(&record);
        experiences.record(record.clone());

        EpisodeReport::new(
            record,
            changes,
            routing_memory.episodes_learned(),
            detailed.then_some(working),
        )
    }

    fn initial_contents(
        &self,
        event: &Event,
    ) -> Vec<(Content, EpistemicKind, Producer, Option<Provenance>)> {
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
                    EpistemicKind::Observation,
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

    fn questions_answered(working: &WorkingMemory) -> bool {
        !working.questions().is_empty()
            && working
                .questions()
                .iter()
                .all(|question| question.is_ground() && working.graph().answers(question))
    }

    /// Modules having valid, unprocessed content they consume, with the
    /// features describing that content.
    fn candidates(
        &self,
        working: &WorkingMemory,
        seen: &[u64],
        previous: Option<&str>,
    ) -> Vec<Candidate> {
        let graph = working.graph();
        let mut candidates = Vec::new();

        for (index, module) in self.registry.modules().iter().enumerate() {
            let descriptor = module.descriptor();
            let mut features = BTreeSet::new();

            for node in graph.valid_nodes() {
                if node.revision() <= seen[index] || !descriptor.consumes(node.content().kind()) {
                    continue;
                }

                if let Some(pattern) = node.content().need()
                    && (!descriptor.supplies(&pattern.signature()) || graph.answers(pattern))
                {
                    continue;
                }

                features.insert(node.content().feature());
            }

            if features.is_empty() {
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

            candidates.push(Candidate::new(
                index,
                descriptor.name().to_owned(),
                features.into_iter().collect(),
            ));
        }

        candidates
    }

    /// Inserts the proposals of a module into the graph, enforcing its
    /// declared contract, then revises beliefs if contradictions appeared.
    fn integrate(
        working: &mut WorkingMemory,
        trace: &mut EpisodeTrace,
        module: &str,
        allowed: impl Fn(EpistemicKind) -> bool,
        output: ModuleOutput,
        step: usize,
    ) -> (Integration, Vec<String>) {
        let mut integration = Integration::default();
        let (proposals, notes) = output.into_parts();

        for proposal in proposals {
            if !allowed(proposal.kind()) {
                integration
                    .rejected
                    .push(format!("undeclared result kind {:?}", proposal.kind()));
                continue;
            }

            let (content, kind, premises, provenance) = proposal.into_parts();
            let feature = content.feature();
            let description = content.to_string();

            match working.graph_mut().insert(
                content,
                kind,
                Producer::Module(module.to_owned()),
                premises,
                provenance,
                step,
            ) {
                Ok(insertion) => {
                    if insertion.created() {
                        integration.produced.push(insertion.id().index());
                        integration.features.insert(feature);
                    }

                    Self::revise(working, insertion.contradictions(), step, trace);
                }
                Err(error) => integration.rejected.push(format!("{description}: {error}")),
            }
        }

        (integration, notes)
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
    /// A need contributes when an answer-supporting statement fulfils it.
    fn assess(
        graph: &ReasoningGraph,
        support: &BTreeSet<NodeId>,
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

        for invocation in invocations {
            if invocation.error().is_some() {
                continue;
            }

            let produced: Vec<NodeId> = invocation
                .produced()
                .iter()
                .map(|index| NodeId::new(*index))
                .collect();

            let outcome = if produced.is_empty() {
                InvocationOutcome::Unproductive
            } else if produced.iter().any(|id| contributes(*id)) {
                InvocationOutcome::Contributed
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
}
