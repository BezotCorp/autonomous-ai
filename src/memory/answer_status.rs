use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AnswerStatus {
    /// At least one valid statement matches the question.
    Proven,
    /// A valid explicit negation answers the question.
    Refuted,
    /// Both a matching statement and its negation are supported.
    Contradictory,
    /// No valid result answers the question: absence of proof only.
    Undetermined,
}
