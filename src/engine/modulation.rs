use serde::{Deserialize, Serialize};

/// Values the reasoning mechanisms take from the internal state of the
/// individual at a given moment. The neutral modulation leaves every
/// mechanism exactly as it is without any latent state (it draws no
/// additional random number), which keeps episodes of the laboratory
/// comparable with those of earlier generations.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Modulation {
    /// Temperature of the posteriors sampled by the selector: above 1 they
    /// are flattened (more exploration), below 1 sharpened.
    temperature: f64,
    /// Factor applied to expected costs.
    cost_factor: f64,
    /// Probability that an applicable influence is admitted in a decision.
    admission: f64,
    /// Probability of abandoning the reasoning before each invocation.
    abandonment: f64,
    /// Factor applied to the number of invocations of an episode.
    invocation_factor: f64,
    /// Probability that imagination goes on to another scene.
    scene_continuation: f64,
    /// Relative weights of the mechanisms of imagination.
    mechanism_weights: [f64; 8],
}

impl Modulation {
    const TEMPERATURE_BOUNDS: (f64, f64) = (0.05, 20.0);
    const FACTOR_BOUNDS: (f64, f64) = (0.01, 100.0);
    /// A slice keeps at least a quarter of its nominal invocations, so that
    /// a reasoning step needing a few invocations remains possible.
    const INVOCATION_BOUNDS: (f64, f64) = (0.25, 4.0);

    pub(crate) fn neutral() -> Self {
        Self {
            temperature: 1.0,
            cost_factor: 1.0,
            admission: 1.0,
            abandonment: 0.0,
            invocation_factor: 1.0,
            scene_continuation: 0.7,
            mechanism_weights: [1.0; 8],
        }
    }

    pub(crate) fn new(
        temperature: f64,
        cost_factor: f64,
        admission: f64,
        abandonment: f64,
        invocation_factor: f64,
        scene_continuation: f64,
        mechanism_weights: [f64; 8],
    ) -> Self {
        let bounded = |value: f64, (low, high): (f64, f64), fallback: f64| {
            if value.is_finite() {
                value.clamp(low, high)
            } else {
                fallback
            }
        };

        Self {
            temperature: bounded(temperature, Self::TEMPERATURE_BOUNDS, 1.0),
            cost_factor: bounded(cost_factor, Self::FACTOR_BOUNDS, 1.0),
            admission: bounded(admission, (0.0, 1.0), 1.0),
            abandonment: bounded(abandonment, (0.0, 1.0), 0.0),
            invocation_factor: bounded(invocation_factor, Self::INVOCATION_BOUNDS, 1.0),
            scene_continuation: bounded(scene_continuation, (0.0, 1.0), 0.7),
            mechanism_weights: mechanism_weights
                .map(|weight| bounded(weight, Self::FACTOR_BOUNDS, 1.0)),
        }
    }

    pub(crate) fn temperature(&self) -> f64 {
        self.temperature
    }

    pub(crate) fn cost_factor(&self) -> f64 {
        self.cost_factor
    }

    pub(crate) fn admission(&self) -> f64 {
        self.admission
    }

    pub(crate) fn abandonment(&self) -> f64 {
        self.abandonment
    }

    pub(crate) fn invocation_factor(&self) -> f64 {
        self.invocation_factor
    }

    pub(crate) fn scene_continuation(&self) -> f64 {
        self.scene_continuation
    }

    pub(crate) fn mechanism_weights(&self) -> &[f64; 8] {
        &self.mechanism_weights
    }
}

impl Default for Modulation {
    fn default() -> Self {
        Self::neutral()
    }
}
