use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::memory::{
    correction::Correction, correction_basis::CorrectionBasis, previous_state::PreviousState,
};

/// Journal entry of an applied correction: what changed, why, on which
/// basis, during which episode, and what it replaced.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct CorrectionRecord {
    id: Uuid,
    applied_at: DateTime<Utc>,
    episode: Uuid,
    correction: Correction,
    basis: CorrectionBasis,
    justification: String,
    previous: PreviousState,
    reverted_at: Option<DateTime<Utc>>,
    revert_reason: Option<String>,
}

impl CorrectionRecord {
    pub(crate) fn new(
        episode: Uuid,
        correction: Correction,
        basis: CorrectionBasis,
        justification: String,
        previous: PreviousState,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            applied_at: Utc::now(),
            episode,
            correction,
            basis,
            justification,
            previous,
            reverted_at: None,
            revert_reason: None,
        }
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn correction(&self) -> &Correction {
        &self.correction
    }

    pub(crate) fn basis(&self) -> &CorrectionBasis {
        &self.basis
    }

    pub(crate) fn previous(&self) -> &PreviousState {
        &self.previous
    }

    pub(crate) fn is_active(&self) -> bool {
        self.reverted_at.is_none()
    }

    pub(crate) fn revert(&mut self, reason: String) {
        self.reverted_at = Some(Utc::now());
        self.revert_reason = Some(reason);
    }
}
