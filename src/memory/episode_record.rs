use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    consumables::{resource_state::ResourceState, resource_usage::ResourceUsage},
    events::event::Event,
    memory::{
        activity::Activity, answer_record::AnswerRecord,
        episode_configuration::EpisodeConfiguration, experiment_request::ExperimentRequest,
        impasse_record::ImpasseRecord, invocation_record::InvocationRecord,
        knowledge_exploitation::KnowledgeExploitation, revision_record::RevisionRecord,
        termination::Termination,
    },
    planning::plan_record::PlanRecord,
    representation::{rule::Rule, statement::Statement},
};

/// Complete trace of one reasoning episode. Failures, refutations and
/// unproductive attempts are kept: they are the raw material of learning.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct EpisodeRecord {
    id: Uuid,
    event_id: String,
    started_at: DateTime<Utc>,
    duration_micros: u64,
    configuration: EpisodeConfiguration,
    questions: Vec<Statement>,
    invocations: Vec<InvocationRecord>,
    revisions: Vec<RevisionRecord>,
    answers: Vec<AnswerRecord>,
    hypotheses_proposed: Vec<Rule>,
    hypotheses_refuted: Vec<Rule>,
    impasses: Vec<ImpasseRecord>,
    termination: Termination,
    notes: Vec<String>,
    plans: Vec<PlanRecord>,
    usage: ResourceUsage,
    resources: BTreeMap<String, ResourceState>,
    exploitation: KnowledgeExploitation,
    event: Option<Event>,
    corrections: Vec<String>,
    experiments: Vec<ExperimentRequest>,
}

impl EpisodeRecord {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: Uuid,
        event_id: String,
        started_at: DateTime<Utc>,
        duration_micros: u64,
        configuration: EpisodeConfiguration,
        questions: Vec<Statement>,
        invocations: Vec<InvocationRecord>,
        revisions: Vec<RevisionRecord>,
        answers: Vec<AnswerRecord>,
        hypotheses_proposed: Vec<Rule>,
        hypotheses_refuted: Vec<Rule>,
        impasses: Vec<ImpasseRecord>,
        termination: Termination,
        notes: Vec<String>,
    ) -> Self {
        Self {
            id,
            event_id,
            started_at,
            duration_micros,
            configuration,
            questions,
            invocations,
            revisions,
            answers,
            hypotheses_proposed,
            hypotheses_refuted,
            impasses,
            termination,
            notes,
            plans: Vec::new(),
            usage: ResourceUsage::default(),
            resources: BTreeMap::new(),
            exploitation: KnowledgeExploitation::default(),
            event: None,
            corrections: Vec::new(),
            experiments: Vec::new(),
        }
    }

    /// Adds what the episode did beyond reasoning: plans, consumption of
    /// external resources and their state at the end, use of the memory.
    pub(crate) fn with_execution(
        mut self,
        plans: Vec<PlanRecord>,
        usage: ResourceUsage,
        resources: BTreeMap<String, ResourceState>,
        exploitation: KnowledgeExploitation,
    ) -> Self {
        self.plans = plans;
        self.usage = usage;
        self.resources = resources;
        self.exploitation = exploitation;
        self
    }

    /// Corrections the episode applied and experiments it requested.
    pub(crate) fn with_reflection(
        mut self,
        corrections: Vec<String>,
        experiments: Vec<ExperimentRequest>,
    ) -> Self {
        self.corrections = corrections;
        self.experiments = experiments;
        self
    }

    pub(crate) fn corrections(&self) -> &[String] {
        &self.corrections
    }

    pub(crate) fn experiments(&self) -> &[ExperimentRequest] {
        &self.experiments
    }

    /// Keeps the event, so that the experience can be replayed.
    pub(crate) fn with_event(mut self, event: Event) -> Self {
        self.event = Some(event);
        self
    }

    pub(crate) fn event(&self) -> Option<&Event> {
        self.event.as_ref()
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn activity(&self) -> Activity {
        self.configuration.activity()
    }

    pub(crate) fn event_id(&self) -> &str {
        &self.event_id
    }

    pub(crate) fn started_at(&self) -> DateTime<Utc> {
        self.started_at
    }

    pub(crate) fn plans(&self) -> &[PlanRecord] {
        &self.plans
    }

    pub(crate) fn usage(&self) -> &ResourceUsage {
        &self.usage
    }

    pub(crate) fn exploitation(&self) -> &KnowledgeExploitation {
        &self.exploitation
    }

    pub(crate) fn duration_micros(&self) -> u64 {
        self.duration_micros
    }

    pub(crate) fn configuration(&self) -> &EpisodeConfiguration {
        &self.configuration
    }

    pub(crate) fn questions(&self) -> &[Statement] {
        &self.questions
    }

    pub(crate) fn invocations(&self) -> &[InvocationRecord] {
        &self.invocations
    }

    pub(crate) fn revisions(&self) -> &[RevisionRecord] {
        &self.revisions
    }

    pub(crate) fn answers(&self) -> &[AnswerRecord] {
        &self.answers
    }

    pub(crate) fn hypotheses_proposed(&self) -> &[Rule] {
        &self.hypotheses_proposed
    }

    pub(crate) fn hypotheses_refuted(&self) -> &[Rule] {
        &self.hypotheses_refuted
    }

    pub(crate) fn impasses(&self) -> &[ImpasseRecord] {
        &self.impasses
    }

    pub(crate) fn termination(&self) -> Termination {
        self.termination
    }
}
