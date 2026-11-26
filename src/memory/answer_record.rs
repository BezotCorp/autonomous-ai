use serde::{Deserialize, Serialize};

use crate::{memory::answer_status::AnswerStatus, representation::statement::Statement};

/// Answer to a question with the justification it rests on.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct AnswerRecord {
    question: Statement,
    status: AnswerStatus,
    answers: Vec<Statement>,
    justification: Vec<String>,
    hypotheses_relied_on: Vec<String>,
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
        }
    }

    pub(crate) fn status(&self) -> AnswerStatus {
        self.status
    }
}
