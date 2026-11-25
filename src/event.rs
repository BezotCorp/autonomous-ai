use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::event_payload::EventPayload;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Event {
    id: String,
    source: String,
    occurred_at: DateTime<Utc>,
    payload: EventPayload,
}

impl Event {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn payload(&self) -> &EventPayload {
        &self.payload
    }
}
