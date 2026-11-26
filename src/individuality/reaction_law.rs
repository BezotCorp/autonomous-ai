use serde::{Deserialize, Serialize};

/// Individual law turning an occurrence (an action, a reaction, a
/// stimulus) into a change of the latent state:
///
/// `Δh = magnitude · strength · gain · (direct + (gate_in · z) · gate_out)`
///
/// The direct part is the same in every state; the gated part depends on
/// the current state `z`, so that the same occurrence may push in opposite
/// directions in different states. `drift` decides, together with the
/// state, whether repeating the occurrence strengthens or weakens its
/// effect (see `Dispositions`). Laws are drawn from a hash of the
/// individual's seed and of the occurrence signature, so that an
/// occurrence never met before receives a law that is new but
/// reproducible, without any table of behaviours.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct ReactionLaw {
    strength: f64,
    direct: Vec<f64>,
    gate_in: Vec<f64>,
    gate_out: Vec<f64>,
    drift: Vec<f64>,
}

impl ReactionLaw {
    pub(crate) fn new(
        strength: f64,
        direct: Vec<f64>,
        gate_in: Vec<f64>,
        gate_out: Vec<f64>,
        drift: Vec<f64>,
    ) -> Self {
        Self {
            strength,
            direct,
            gate_in,
            gate_out,
            drift,
        }
    }

    #[cfg(test)]
    pub(crate) fn direct(&self) -> &[f64] {
        &self.direct
    }

    /// Change of the pre-activation for a given state, magnitude and gain.
    pub(crate) fn impulse(&self, latent: &[f64], magnitude: f64, gain: f64) -> Vec<f64> {
        let gate: f64 = self.gate_in.iter().zip(latent).map(|(a, b)| a * b).sum();
        let factor = magnitude * self.strength * gain;

        self.direct
            .iter()
            .zip(&self.gate_out)
            .map(|(direct, out)| factor * (direct + gate * out))
            .collect()
    }

    /// Projection of the state deciding how repetition changes the gain.
    pub(crate) fn drift(&self, latent: &[f64]) -> f64 {
        self.drift.iter().zip(latent).map(|(a, b)| a * b).sum()
    }
}
