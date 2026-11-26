use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Temporal bookkeeping of an existence. Lived time is the time during
/// which the individual existed while its process ran (wall-clock or
/// virtual); dormant time is the time between two sessions, while the
/// process was stopped, during which nothing cognitive happened.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct LifeClock {
    lived: f64,
    dormant: f64,
    sessions: u64,
    last_wall: DateTime<Utc>,
}

impl LifeClock {
    pub(crate) fn new() -> Self {
        Self {
            lived: 0.0,
            dormant: 0.0,
            sessions: 0,
            last_wall: Utc::now(),
        }
    }

    pub(crate) fn lived(&self) -> f64 {
        self.lived
    }

    pub(crate) fn dormant(&self) -> f64 {
        self.dormant
    }

    pub(crate) fn sessions(&self) -> u64 {
        self.sessions
    }

    /// Opens a session; returns the wall-clock time elapsed since the
    /// existence was last saved.
    pub(crate) fn open_session(&mut self) -> f64 {
        self.sessions += 1;

        let now = Utc::now();
        let elapsed = (now - self.last_wall).num_milliseconds().max(0) as f64 / 1000.0;

        self.last_wall = now;
        elapsed
    }

    pub(crate) fn sleep_through(&mut self, seconds: f64) {
        if seconds.is_finite() && seconds > 0.0 {
            self.dormant += seconds;
        }
    }

    pub(crate) fn advance(&mut self, seconds: f64) {
        if seconds.is_finite() && seconds > 0.0 {
            self.lived += seconds;
        }

        self.last_wall = Utc::now();
    }
}

impl Default for LifeClock {
    fn default() -> Self {
        Self::new()
    }
}
