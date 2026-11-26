use std::sync::{Arc, Mutex, MutexGuard};

use uuid::Uuid;

use crate::world::{
    environment::Environment, message::Message, portal_state::PortalState,
    transmission::Transmission, transmission_status::TransmissionStatus,
};

/// The access of one present existence to its environment: what its
/// capabilities may perceive or emit there, and what reaches it. Several
/// portals share one environment.
///
/// Environment time is the existence's lived time minus an offset fixed
/// when its session begins: every existence keeps its own lived time, the
/// environment its common one.
#[derive(Clone)]
pub(crate) struct Portal {
    environment: Arc<Mutex<Environment>>,
    handle: String,
    state: Arc<Mutex<PortalState>>,
}

impl Portal {
    pub(crate) fn new(environment: Arc<Mutex<Environment>>, handle: String, lived: f64) -> Self {
        let clock = Self::lock(&environment).clock();

        Self {
            environment,
            handle,
            state: Arc::new(Mutex::new(PortalState::new(lived - clock, clock))),
        }
    }

    fn lock(environment: &Arc<Mutex<Environment>>) -> MutexGuard<'_, Environment> {
        // A poisoned lock means another existence panicked while holding
        // it; the environment data stays usable.
        environment
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn state(&self) -> MutexGuard<'_, PortalState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Environment time corresponding to a lived time of the existence.
    pub(crate) fn time_of(&self, lived: f64) -> f64 {
        lived - self.state().offset()
    }

    /// Lived time of the existence corresponding to an environment time.
    pub(crate) fn lived_at(&self, time: f64) -> f64 {
        time + self.state().offset()
    }

    /// The existence reached this lived time.
    pub(crate) fn follow(&self, lived: f64) {
        let time = self.time_of(lived);

        self.state().set_now(time);
    }

    pub(crate) fn now(&self) -> f64 {
        self.state().now()
    }

    /// Other presences perceptible now.
    pub(crate) fn perceive(&self) -> Vec<String> {
        let now = self.now();

        Self::lock(&self.environment).perceptible(&self.handle, now)
    }

    /// Emits a message now.
    pub(crate) fn transmit(&self, to: &str, message: Message) -> (Uuid, TransmissionStatus) {
        let now = self.now();
        let (id, status) = Self::lock(&self.environment).transmit(&self.handle, to, now, message);

        self.state().emit(id, to.to_owned(), status.clone());
        (id, status)
    }

    /// Emissions made since the last call (for the existence's timeline).
    pub(crate) fn take_emitted(&self) -> Vec<(Uuid, String, TransmissionStatus)> {
        self.state().take_emitted()
    }

    /// Environment time at which the next message for this existence can
    /// be received, if one is on its way.
    pub(crate) fn next_arrival(&self) -> Option<f64> {
        Self::lock(&self.environment).next_arrival(&self.handle)
    }

    /// Messages that can be received now.
    pub(crate) fn receive(&self) -> Vec<Transmission> {
        let now = self.now();

        Self::lock(&self.environment).deliver(&self.handle, now)
    }
}
