use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TransmissionStatus {
    /// On its way; it can be received from `deliver_at` on.
    Pending,
    /// Handed to the recipient as a stimulus, at that environment time.
    Delivered { at: f64 },
    /// The destination does not exist here (or left): nothing arrives.
    Undeliverable { reason: String },
}
