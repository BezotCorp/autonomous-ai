use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{memory::answer_status::AnswerStatus, representation::statement::Statement};

/// Answer to a question with the justification it rests on, and how
/// current it is: an answer resting on recollections states what held when
/// they were last verified (`as_of`), with the probability that it no
/// longer holds now (`staleness`).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct AnswerRecord {
    question: Statement,
    status: AnswerStatus,
    answers: Vec<Statement>,
    justification: Vec<String>,
    hypotheses_relied_on: Vec<String>,
    staleness: f64,
    as_of: Option<DateTime<Utc>>,
    recollections: usize,
}

impl AnswerRecord {
    pub(crate) fn new(
        question: Statement,
        status: AnswerStatus,
        answers: Vec<Statement>,
        justification: Vec<String>,
        hypotheses_relied_on: Vec<String>,
    ) -> Self {
        Self {
            question,
            status,
            answers,
            justification,
            hypotheses_relied_on,
            staleness: 0.0,
            as_of: None,
            recollections: 0,
        }
    }

    pub(crate) fn dated(
        mut self,
        staleness: f64,
        as_of: Option<DateTime<Utc>>,
        recollections: usize,
    ) -> Self {
        self.staleness = staleness;
        self.as_of = as_of;
        self.recollections = recollections;
        self
    }

    pub(crate) fn staleness(&self) -> f64 {
        self.staleness
    }

    /// Whether the answer rests only on what is known to hold now.
    pub(crate) fn is_current(&self) -> bool {
        self.staleness == 0.0
    }

    pub(crate) fn question(&self) -> &Statement {
        &self.question
    }

    pub(crate) fn answers(&self) -> &[Statement] {
        &self.answers
    }

    pub(crate) fn status(&self) -> AnswerStatus {
        self.status
    }
}
