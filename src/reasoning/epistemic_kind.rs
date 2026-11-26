use serde::{Deserialize, Serialize};

/// Epistemic status of a result: how it came to be believed.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EpistemicKind {
    /// Directly perceived from an event or acquired from an external source.
    Observation,
    /// Reproduced, during a dream, from data recorded in a past acquisition.
    /// It stands for an observation inside the dream only: it is never a
    /// new, independent observation of the world.
    Replay,
    /// Provided by the initial resources.
    Premise,
    /// Recalled from the persistent knowledge memory.
    Recollection,
    /// Conjecture that has not been established.
    Hypothesis,
    /// Logically derived from other results.
    Deduction,
    /// Obtained by an exact computation over other results.
    Computation,
    /// Information the intelligence lacks.
    Need,
}

impl EpistemicKind {
    /// Contents standing for what the environment showed.
    pub(crate) fn is_observed(self) -> bool {
        matches!(self, Self::Observation | Self::Replay)
    }

    /// Results that may be withdrawn when a contradiction is traced back to them.
    pub(crate) fn is_defeasible(self) -> bool {
        matches!(self, Self::Hypothesis | Self::Recollection)
    }
}
