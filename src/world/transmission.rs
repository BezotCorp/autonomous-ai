use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::world::{message::Message, transmission_status::TransmissionStatus};

/// One message on its way through the environment, dated in environment
/// time: it can never be received before `emitted_at + latency`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Transmission {
    id: Uuid,
    from: String,
    to: String,
    emitted_at: f64,
    deliver_at: f64,
    message: Message,
    status: TransmissionStatus,
}

impl Transmission {
    pub(crate) fn new(
        id: Uuid,
        from: String,
        to: String,
        emitted_at: f64,
        deliver_at: f64,
        message: Message,
        status: TransmissionStatus,
    ) -> Self {
        Self {
            id,
            from,
            to,
            emitted_at,
            deliver_at,
            message,
            status,
        }
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn from(&self) -> &str {
        &self.from
    }

    pub(crate) fn to(&self) -> &str {
        &self.to
    }

    pub(crate) fn emitted_at(&self) -> f64 {
        self.emitted_at
    }

    pub(crate) fn deliver_at(&self) -> f64 {
        self.deliver_at
    }

    pub(crate) fn message(&self) -> &Message {
        &self.message
    }

    pub(crate) fn status(&self) -> &TransmissionStatus {
        &self.status
    }

    pub(crate) fn deliver(&mut self, at: f64) {
        self.status = TransmissionStatus::Delivered { at };
    }
}
