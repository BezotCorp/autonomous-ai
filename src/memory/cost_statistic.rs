use serde::{Deserialize, Serialize};

/// Measured execution cost of a module.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub(crate) struct CostStatistic {
    invocations: u64,
    total_micros: u64,
}

impl CostStatistic {
    pub(crate) fn observe(&mut self, micros: u64) {
        self.invocations += 1;
        self.total_micros = self.total_micros.saturating_add(micros);
    }

    /// Mean duration in seconds, or `None` before any measurement.
    pub(crate) fn mean_seconds(&self) -> Option<f64> {
        (self.invocations > 0)
            .then(|| self.total_micros as f64 / self.invocations as f64 / 1_000_000.0)
    }
}
