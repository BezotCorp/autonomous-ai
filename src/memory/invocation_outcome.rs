use serde::{Deserialize, Serialize};

/// Verifiable outcome of a module invocation, assessed at the end of the
/// episode rather than when the module returned.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InvocationOutcome {
    /// At least one result supports an answer to a question.
    Contributed,
    /// Without supporting an answer, it made verifiable progress: it
    /// satisfied a need that was open, or formulated a need that was
    /// satisfied afterwards.
    Advanced,
    /// Results remained valid but did not support any answer.
    Retained,
    /// Every result was later invalidated.
    Invalidated,
    /// Nothing new was produced.
    Unproductive,
    /// The module failed technically.
    Failed,
}

impl InvocationOutcome {
    /// Success criterion used by the routing statistics. Supporting an
    /// answer or satisfying a need are verifiable successes even when the
    /// episode as a whole fails; merely keeping valid results only counts
    /// when no question was asked.
    pub(crate) fn is_success(self, questions_asked: bool) -> bool {
        match self {
            Self::Contributed | Self::Advanced => true,
            Self::Retained => !questions_asked,
            Self::Invalidated | Self::Unproductive | Self::Failed => false,
        }
    }
}
