use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{living::concern_status::ConcernStatus, representation::statement::Statement};

/// Something that occupies the individual beyond the moment it arose: an
/// impasse, a request left unanswered, a contradiction, a need met in
/// imagination. A concern has a salience, which fades with time at a rate
/// the latent state modulates and grows again whenever something related
/// happens or the individual attends to it. It begins unconscious and may
/// surface. Nothing requires a concern to be resolved, nor its
/// persistence to be useful.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Concern {
    id: Uuid,
    subject: Statement,
    origin: String,
    salience: f64,
    conscious: bool,
    formed_at: f64,
    last_activated: f64,
    activations: u64,
    status: ConcernStatus,
}

impl Concern {
    pub(crate) const MAXIMUM_SALIENCE: f64 = 50.0;

    pub(crate) fn new(subject: Statement, origin: String, salience: f64, lived: f64) -> Self {
        Self {
            id: Uuid::now_v7(),
            subject,
            origin,
            salience: Self::bounded(salience),
            conscious: false,
            formed_at: lived,
            last_activated: lived,
            activations: 1,
            status: ConcernStatus::Active,
        }
    }

    fn bounded(salience: f64) -> f64 {
        if salience.is_finite() {
            salience.clamp(0.0, Self::MAXIMUM_SALIENCE)
        } else {
            0.0
        }
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn subject(&self) -> &Statement {
        &self.subject
    }

    pub(crate) fn origin(&self) -> &str {
        &self.origin
    }

    pub(crate) fn salience(&self) -> f64 {
        self.salience
    }

    pub(crate) fn is_conscious(&self) -> bool {
        self.conscious
    }

    pub(crate) fn is_active(&self) -> bool {
        self.status == ConcernStatus::Active
    }

    pub(crate) fn status(&self) -> ConcernStatus {
        self.status
    }

    pub(crate) fn activations(&self) -> u64 {
        self.activations
    }

    pub(crate) fn activate(&mut self, gain: f64, lived: f64) {
        self.salience = Self::bounded(self.salience + gain);
        self.last_activated = lived;
        self.activations += 1;
    }

    /// Attention multiplies the salience, up or down.
    pub(crate) fn attend(&mut self, factor: f64, lived: f64) {
        self.salience = Self::bounded(self.salience * factor);
        self.last_activated = lived;
        self.activations += 1;
    }

    pub(crate) fn fade(&mut self, factor: f64) {
        self.salience = Self::bounded(self.salience * factor);
    }

    pub(crate) fn surface(&mut self) {
        self.conscious = true;
    }

    pub(crate) fn conclude(&mut self, status: ConcernStatus) {
        self.status = status;
    }
}
