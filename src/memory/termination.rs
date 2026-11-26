use serde::{Deserialize, Serialize};

/// Why a reasoning episode stopped.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Termination {
    /// Every question received a definite answer.
    QuestionsAnswered,
    /// No module had unprocessed relevant input any more.
    Quiescent,
    /// No candidate module was expected to be useful enough.
    NothingWorthTrying,
    InvocationBudgetExhausted,
    TimeBudgetExhausted,
}
