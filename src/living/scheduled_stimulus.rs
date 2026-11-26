use serde::{Deserialize, Serialize};

use crate::events::event::Event;

/// A stimulus an experimenter plans to deliver, at a lived time counted
/// from the start of the session.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ScheduledStimulus {
    at: f64,
    event: Event,
}

impl ScheduledStimulus {
    pub(crate) fn at(&self) -> f64 {
        self.at
    }

    pub(crate) fn event(&self) -> &Event {
        &self.event
    }
}
