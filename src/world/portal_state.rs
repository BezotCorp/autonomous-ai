use uuid::Uuid;

use crate::world::transmission_status::TransmissionStatus;

/// Mutable part of a portal: the offset between the existence's lived time
/// and environment time, the environment time reached, and the emissions
/// not yet recorded in the existence's timeline.
#[derive(Default)]
pub(crate) struct PortalState {
    offset: f64,
    now: f64,
    emitted: Vec<(Uuid, String, TransmissionStatus)>,
}

impl PortalState {
    pub(crate) fn new(offset: f64, now: f64) -> Self {
        Self {
            offset,
            now,
            emitted: Vec::new(),
        }
    }

    pub(crate) fn offset(&self) -> f64 {
        self.offset
    }

    pub(crate) fn now(&self) -> f64 {
        self.now
    }

    pub(crate) fn set_now(&mut self, now: f64) {
        self.now = now;
    }

    pub(crate) fn emit(&mut self, id: Uuid, to: String, status: TransmissionStatus) {
        self.emitted.push((id, to, status));
    }

    pub(crate) fn take_emitted(&mut self) -> Vec<(Uuid, String, TransmissionStatus)> {
        std::mem::take(&mut self.emitted)
    }
}
