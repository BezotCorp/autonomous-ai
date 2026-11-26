use serde::{Deserialize, Serialize};

/// Comparison of a dream with the experience it replayed.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DreamVerdict {
    /// Same answers, with less effort or consumption.
    Cheaper,
    /// More definite answers (in simulation only, until verified).
    MoreAnswers,
    /// Same answers and comparable effort.
    Equivalent,
    /// Fewer definite answers or different ones.
    Worse,
    /// The replay could not be carried out.
    Failed,
}
