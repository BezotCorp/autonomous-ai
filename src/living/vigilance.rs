use serde::{Deserialize, Serialize};

/// Global configuration of the functioning: awake, the conscious may act,
/// introspect, simulate and daydream; asleep, it is closed (stimuli wait,
/// conscious activities are suspended) while the unconscious goes on and
/// dreams may arise. Neither state is imposed: each is left at a rate the
/// latent state modulates.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Vigilance {
    #[default]
    Awake,
    Asleep,
}

impl Vigilance {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Awake => "awake",
            Self::Asleep => "asleep",
        }
    }
}
