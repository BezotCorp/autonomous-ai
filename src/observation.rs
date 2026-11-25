use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::observation_payload::ObservationPayload;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Observation {
    id: Uuid,
    event_id: String,
    observed_at: DateTime<Utc>,
    payload: ObservationPayload,
}

impl Observation {
    pub(crate) fn is_failure(&self) -> bool {
        matches!(&self.payload, ObservationPayload::Failed { .. })
    }

    pub(crate) fn new(event_id: String, payload: ObservationPayload) -> Self {
        Self {
            id: Uuid::now_v7(),
            event_id,
            observed_at: Utc::now(),
            payload,
        }
    }
}
