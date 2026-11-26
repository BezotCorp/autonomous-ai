use serde::{Deserialize, Serialize};

/// Cognitive activity an episode belongs to. All run on the same engine;
/// they differ by where their goals come from, by whether they are
/// accessible to the conscious, and by what they may affect.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Activity {
    /// Acting on the environment, usually after a stimulus (conscious).
    #[default]
    Action,
    /// Examining and correcting the cognition (unconscious).
    Reflection,
    /// Examining oneself: one's state as perceived, concerns, activities,
    /// beliefs (conscious).
    Introspection,
    /// Imagining directed towards a situation or a concern (conscious).
    Simulation,
    /// Imagining freely while awake (conscious).
    Reverie,
    /// Imagining freely while asleep (unconscious).
    Dream,
}

impl Activity {
    pub(crate) const ALL: [Self; 6] = [
        Self::Action,
        Self::Reflection,
        Self::Introspection,
        Self::Simulation,
        Self::Reverie,
        Self::Dream,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Action => "action",
            Self::Reflection => "reflection",
            Self::Introspection => "introspection",
            Self::Simulation => "simulation",
            Self::Reverie => "reverie",
            Self::Dream => "dream",
        }
    }

    pub(crate) fn parse(label: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|activity| activity.label() == label)
    }

    /// Unfolds in imagination: never reaches the environment, never
    /// produces facts about the world.
    pub(crate) fn is_imaginary(self) -> bool {
        matches!(self, Self::Simulation | Self::Reverie | Self::Dream)
    }

    /// Accessible to the conscious (and open to the influences of the
    /// unconscious).
    pub(crate) fn is_conscious(self) -> bool {
        matches!(
            self,
            Self::Action | Self::Introspection | Self::Simulation | Self::Reverie
        )
    }
}
