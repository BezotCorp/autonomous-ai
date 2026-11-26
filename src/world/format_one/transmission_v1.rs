use serde::Deserialize;
use uuid::Uuid;

use crate::world::{
    format_one::message_v1::MessageV1, transmission::Transmission,
    transmission_status::TransmissionStatus,
};

/// A transmission as format 1 of environments stored it. Read only.
#[derive(Deserialize)]
pub(crate) struct TransmissionV1 {
    id: Uuid,
    from: String,
    to: String,
    emitted_at: f64,
    deliver_at: f64,
    message: MessageV1,
    status: TransmissionStatus,
}

impl TransmissionV1 {
    pub(crate) fn migrate(self) -> Transmission {
        Transmission::new(
            self.id,
            self.from,
            self.to,
            self.emitted_at,
            self.deliver_at,
            self.message.migrate(),
            self.status,
        )
    }
}
