use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{events::event_payload::EventPayload, representation::statement::Statement};

/// Something that happened, received from the environment, optionally
/// accompanied by questions the intelligence is asked to answer.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Event {
    id: String,
    source: String,
    occurred_at: DateTime<Utc>,
    payload: EventPayload,
    #[serde(default)]
    questions: Vec<Statement>,
}

impl Event {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn source(&self) -> &str {
        &self.source
    }

    pub(crate) fn payload(&self) -> &EventPayload {
        &self.payload
    }

    pub(crate) fn questions(&self) -> &[Statement] {
        &self.questions
    }
}
