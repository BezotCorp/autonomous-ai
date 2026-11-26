use serde::{Deserialize, Serialize};

use crate::memory::activity::Activity;

/// How an activity ended.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ActivityEnding {
    /// It reached what it was about (answers, an unfolding, a quiescence).
    Completed,
    /// The reasoner gave up during it.
    Abandoned,
    /// The internal dynamics interrupted it.
    Interrupted,
    /// It drifted into another kind of activity.
    Transformed(Activity),
    /// Suspended, it faded away without being resumed.
    Dissolved,
    /// It reached the technical bound on the slices of one activity.
    Bounded,
}

impl ActivityEnding {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Abandoned => "abandoned",
            Self::Interrupted => "interrupted",
            Self::Transformed(_) => "transformed",
            Self::Dissolved => "dissolved",
            Self::Bounded => "bounded",
        }
    }
}
