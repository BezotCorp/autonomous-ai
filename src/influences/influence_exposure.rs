use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::memory::{activity::Activity, invocation_outcome::InvocationOutcome};

/// What became of an influence in one episode: whether its content entered
/// the reasoning, which decisions it changed (compared with the choice
/// that would have been made without it) and with what outcome, whether
/// its content supported an answer or was refuted.
///
/// Presence alone only shows a correlation; a decision it changed with a
/// verified outcome is an observed contribution; a demonstration requires
/// replaying the episode without it.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct InfluenceExposure {
    influence: Uuid,
    episode: Uuid,
    activity: Activity,
    injected: usize,
    decisive: Vec<(usize, InvocationOutcome)>,
    supported_answer: bool,
    refuted: bool,
    definite_answers: usize,
}

impl InfluenceExposure {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        influence: Uuid,
        episode: Uuid,
        activity: Activity,
        injected: usize,
        decisive: Vec<(usize, InvocationOutcome)>,
        supported_answer: bool,
        refuted: bool,
        definite_answers: usize,
    ) -> Self {
        Self {
            influence,
            episode,
            activity,
            injected,
            decisive,
            supported_answer,
            refuted,
            definite_answers,
        }
    }

    pub(crate) fn influence(&self) -> Uuid {
        self.influence
    }

    pub(crate) fn episode(&self) -> Uuid {
        self.episode
    }

    pub(crate) fn is_real(&self) -> bool {
        !self.activity.is_imaginary()
    }

    pub(crate) fn decisive(&self) -> &[(usize, InvocationOutcome)] {
        &self.decisive
    }

    pub(crate) fn supported_answer(&self) -> bool {
        self.supported_answer
    }

    pub(crate) fn refuted(&self) -> bool {
        self.refuted
    }
}
