use serde::{Deserialize, Serialize};

use crate::individuality::constitution::Constitution;

/// Current point of the latent space. It is kept as a pre-activation `h`,
/// bounded in `[−BOUND, BOUND]`; what mechanisms read is `z = tanh(h)`,
/// in the open hypercube `(−1, 1)^D`. The dimensions have no meaning of
/// their own: only the individual laws reading and moving them do.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct LatentState {
    pre: Vec<f64>,
    repairs: u64,
}

impl LatentState {
    pub(crate) const BOUND: f64 = 6.0;
    /// Levels at which a sensation is perceived as low or high.
    const SENSATION_LEVEL: f64 = 0.3;

    pub(crate) fn new(initial: &[f64]) -> Self {
        let mut state = Self {
            pre: initial.to_vec(),
            repairs: 0,
        };

        state.enforce(initial);
        state
    }

    pub(crate) fn pre(&self) -> &[f64] {
        &self.pre
    }

    pub(crate) fn activation(&self) -> Vec<f64> {
        self.pre.iter().map(|value| value.tanh()).collect()
    }

    pub(crate) fn dimensions(&self) -> usize {
        self.pre.len()
    }

    /// Non-finite values that had to be repaired: a software defect, never
    /// a cognitive phenomenon, counted so that it cannot go unnoticed.
    pub(crate) fn repairs(&self) -> u64 {
        self.repairs
    }

    pub(crate) fn set_pre(&mut self, values: Vec<f64>, fallback: &[f64]) {
        self.pre = values;
        self.enforce(fallback);
    }

    pub(crate) fn shift(&mut self, change: &[f64], fallback: &[f64]) {
        for (value, change) in self.pre.iter_mut().zip(change) {
            *value += change;
        }

        self.enforce(fallback);
    }

    /// Keeps every component finite and within bounds; a non-finite value
    /// is replaced by the fallback (the resting point) and counted.
    fn enforce(&mut self, fallback: &[f64]) {
        for (index, value) in self.pre.iter_mut().enumerate() {
            if !value.is_finite() {
                *value = fallback.get(index).copied().unwrap_or(0.0);
                self.repairs += 1;
            }

            *value = value.clamp(-Self::BOUND, Self::BOUND);
        }
    }

    /// Anonymous projections of the state that introspection perceives,
    /// each as low, middle or high.
    pub(crate) fn sensations(&self, constitution: &Constitution) -> Vec<(usize, &'static str)> {
        let latent = self.activation();

        constitution
            .sensations()
            .iter()
            .enumerate()
            .map(|(index, direction)| {
                let norm = direction
                    .iter()
                    .map(|value| value * value)
                    .sum::<f64>()
                    .sqrt()
                    .max(f64::MIN_POSITIVE);
                let projection = direction
                    .iter()
                    .zip(&latent)
                    .map(|(a, b)| a * b)
                    .sum::<f64>()
                    / norm;

                (
                    index + 1,
                    if projection < -Self::SENSATION_LEVEL {
                        "low"
                    } else if projection > Self::SENSATION_LEVEL {
                        "high"
                    } else {
                        "middle"
                    },
                )
            })
            .collect()
    }
}
