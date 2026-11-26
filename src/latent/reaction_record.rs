use serde::{Deserialize, Serialize};

/// What one kind of occurrence did to the latent state so far.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ReactionRecord {
    count: u64,
    first_seconds: f64,
    last_seconds: f64,
    total_impulse: Vec<f64>,
    total_norm: f64,
}

impl ReactionRecord {
    pub(crate) fn new(lived_seconds: f64, dimensions: usize) -> Self {
        Self {
            count: 0,
            first_seconds: lived_seconds,
            last_seconds: lived_seconds,
            total_impulse: vec![0.0; dimensions],
            total_norm: 0.0,
        }
    }

    pub(crate) fn observe(&mut self, impulse: &[f64], lived_seconds: f64) {
        self.count += 1;
        self.last_seconds = lived_seconds;
        self.total_norm += impulse
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt();

        for (total, value) in self.total_impulse.iter_mut().zip(impulse) {
            *total += value;
        }
    }

    pub(crate) fn count(&self) -> u64 {
        self.count
    }

    /// Mean size of the change it caused.
    pub(crate) fn mean_norm(&self) -> f64 {
        self.total_norm / self.count.max(1) as f64
    }

    /// Mean change it caused, direction included.
    pub(crate) fn mean_impulse(&self) -> Vec<f64> {
        self.total_impulse
            .iter()
            .map(|value| value / self.count.max(1) as f64)
            .collect()
    }
}
