use serde::{Deserialize, Serialize};

/// Average weighting recent observations more: a plain mean over the first
/// observations, then an exponential moving average, so that estimates
/// follow changes (e.g. a module becoming cheaper once its data is stored).
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub(crate) struct MovingAverage {
    value: f64,
    observations: u64,
}

impl MovingAverage {
    const RECENT_WEIGHT: f64 = 0.2;

    pub(crate) fn observe(&mut self, value: f64) {
        self.observations += 1;

        let weight = (1.0 / self.observations as f64).max(Self::RECENT_WEIGHT);

        self.value += weight * (value - self.value);
    }

    pub(crate) fn value(&self) -> Option<f64> {
        (self.observations > 0).then_some(self.value)
    }
}
