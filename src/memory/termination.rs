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
    /// Every candidate module required a resource observed to be exhausted.
    ResourcesExhausted,
    InvocationBudgetExhausted,
    TimeBudgetExhausted,
    /// The reasoner gave up, under the modulation of its internal state,
    /// while the reasoning could have gone on.
    Abandoned,
}

impl Termination {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::QuestionsAnswered => "questions_answered",
            Self::Quiescent => "quiescent",
            Self::NothingWorthTrying => "nothing_worth_trying",
            Self::ResourcesExhausted => "resources_exhausted",
            Self::InvocationBudgetExhausted => "invocation_budget_exhausted",
            Self::TimeBudgetExhausted => "time_budget_exhausted",
            Self::Abandoned => "abandoned",
        }
    }
}
