use serde::{Deserialize, Serialize};

use crate::memory::moving_average::MovingAverage;

/// Measured execution cost of a module: overall, and recently, since the
/// cost of a module changes with what it already stored.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub(crate) struct CostStatistic {
    invocations: u64,
    total_micros: u64,
    recent_seconds: MovingAverage,
}

impl CostStatistic {
    pub(crate) fn observe(&mut self, micros: u64) {
        self.invocations += 1;
        self.total_micros = self.total_micros.saturating_add(micros);
        self.recent_seconds.observe(micros as f64 / 1_000_000.0);
    }

    /// Duration expected from the recent invocations, or `None` before any.
    pub(crate) fn expected_seconds(&self) -> Option<f64> {
        self.recent_seconds.value()
    }

    pub(crate) fn invocations(&self) -> u64 {
        self.invocations
    }
}
