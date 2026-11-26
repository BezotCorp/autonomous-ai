use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::events::event::Event;

/// An event received from outside. It is a stimulation, not an instruction:
/// it waits with a salience that fades, and may give rise to an action,
/// wake the individual, or be neglected.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Stimulus {
    id: Uuid,
    event: Event,
    received_at: f64,
    salience: f64,
}

impl Stimulus {
    pub(crate) fn new(event: Event, received_at: f64, salience: f64) -> Self {
        Self {
            id: Uuid::now_v7(),
            event,
            received_at,
            salience: if salience.is_finite() {
                salience.clamp(0.0, 100.0)
            } else {
                0.0
            },
        }
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn event(&self) -> &Event {
        &self.event
    }

    pub(crate) fn salience(&self) -> f64 {
        self.salience
    }

    pub(crate) fn fade(&mut self, factor: f64) {
        if factor.is_finite() {
            self.salience = (self.salience * factor).max(0.0);
        }
    }
}
