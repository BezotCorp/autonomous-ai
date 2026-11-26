use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::living::life_event::LifeEvent;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct TimelineEntry {
    lived: f64,
    wall: DateTime<Utc>,
    event: LifeEvent,
}

impl TimelineEntry {
    pub(crate) fn new(lived: f64, event: LifeEvent) -> Self {
        Self {
            lived,
            wall: Utc::now(),
            event,
        }
    }

    pub(crate) fn lived(&self) -> f64 {
        self.lived
    }

    pub(crate) fn event(&self) -> &LifeEvent {
        &self.event
    }
}
