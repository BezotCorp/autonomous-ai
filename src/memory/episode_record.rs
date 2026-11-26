use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    memory::{
        answer_record::AnswerRecord, episode_configuration::EpisodeConfiguration,
        invocation_record::InvocationRecord, revision_record::RevisionRecord,
        termination::Termination,
    },
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
    open_needs: Vec<Statement>,
    termination: Termination,
    notes: Vec<String>,
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
        open_needs: Vec<Statement>,
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
            open_needs,
            termination,
            notes,
        }
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

    pub(crate) fn termination(&self) -> Termination {
        self.termination
    }
}
