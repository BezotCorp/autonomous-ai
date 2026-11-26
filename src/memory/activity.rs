use serde::{Deserialize, Serialize};

/// Cognitive activity an episode belongs to. All three run on the same
/// engine; they differ by where their goals come from and by what they
/// may affect.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Activity {
    /// Solving a problem received from the environment.
    #[default]
    Action,
    /// Examining and correcting the cognition itself (the unconscious).
    Reflection,
    /// Replaying or exploring past experiences internally, without any
    /// effect on, or request to, the environment.
    Dream,
}

impl Activity {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Action => "action",
            Self::Reflection => "reflection",
            Self::Dream => "dream",
        }
    }
}
