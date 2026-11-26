use serde::{Deserialize, Serialize};

use crate::mathematics::random_source::RandomSource;

/// Scales drawn first at birth, from which the laws of the individual are
/// then drawn. Drawing them per individual (instead of fixing them) is what
/// lets individuals differ in kind and not only in detail: one may have a
/// latent space that relaxes in seconds and another one in hours, one a
/// weak internal coupling (calm, decaying dynamics) and another one a
/// coupling strong enough for self-sustained oscillations or chaos.
///
/// Every distribution is log-normal (median, deviation of the logarithm)
/// unless stated otherwise; the medians are orders of magnitude, not
/// measured values.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct ConstitutionScales {
    /// Median relaxation time of the latent dimensions, in seconds
    /// (median 600 s, deviation 0.7).
    time_scale: f64,
    /// Spread of the relaxation times around that median (median 0.8,
    /// deviation 0.3).
    time_spread: f64,
    /// Gain of the internal coupling (median 0.9, deviation 0.4). Around 1
    /// and above, the recurrent dynamics may stop decaying to a fixed point.
    coupling_gain: f64,
    /// Scale of the resting point (median 0.5, deviation 0.5).
    baseline_scale: f64,
    /// Intensity of the spontaneous fluctuations (median 0.3, deviation 0.6).
    noise: f64,
    /// Scale of the effect of occurrences (median 0.4, deviation 0.5).
    impulse_scale: f64,
    /// Weight of the state-dependent part of reactions (median 1, deviation 0.5).
    gating: f64,
    /// Share of the reaction to an occurrence that it has in common with
    /// occurrences of the same family (uniform in [0.2, 0.9]).
    family_share: f64,
    /// Scale of the sensitivities of channels to the latent (median 1.5,
    /// deviation 0.5).
    sensitivity_scale: f64,
    /// Spread of the biases of channels around their centre (median 1,
    /// deviation 0.3).
    bias_spread: f64,
    /// Plasticity: change of the logarithm of a reaction gain per
    /// occurrence (median 0.05, deviation 1).
    gain_plasticity: f64,
    /// Plasticity of the internal coupling, per hour (median 0.02, deviation 1).
    coupling_plasticity: f64,
    /// Plasticity of the resting point, per hour (median 0.01, deviation 1).
    baseline_plasticity: f64,
    /// Plasticity of the sensitivities of channels, per engagement of the
    /// mechanism (median 0.1, deviation 1).
    habit_plasticity: f64,
    /// Sign and strength of the co-activity rule changing the internal
    /// coupling (uniform in [−1, 1]): positive, dimensions active together
    /// come to reinforce each other; negative, they come to inhibit each
    /// other.
    coupling_polarity: f64,
}

impl ConstitutionScales {
    pub(crate) fn draw(random: &mut RandomSource) -> Self {
        Self {
            time_scale: random.log_normal(600.0, 0.7),
            time_spread: random.log_normal(0.8, 0.3),
            coupling_gain: random.log_normal(0.9, 0.4),
            baseline_scale: random.log_normal(0.5, 0.5),
            noise: random.log_normal(0.3, 0.6),
            impulse_scale: random.log_normal(0.4, 0.5),
            gating: random.log_normal(1.0, 0.5),
            family_share: 0.2 + 0.7 * random.uniform(),
            sensitivity_scale: random.log_normal(1.5, 0.5),
            bias_spread: random.log_normal(1.0, 0.3),
            gain_plasticity: random.log_normal(0.05, 1.0),
            coupling_plasticity: random.log_normal(0.02, 1.0),
            baseline_plasticity: random.log_normal(0.01, 1.0),
            habit_plasticity: random.log_normal(0.1, 1.0),
            coupling_polarity: 2.0 * random.uniform() - 1.0,
        }
    }

    pub(crate) fn time_scale(&self) -> f64 {
        self.time_scale
    }

    pub(crate) fn time_spread(&self) -> f64 {
        self.time_spread
    }

    pub(crate) fn coupling_gain(&self) -> f64 {
        self.coupling_gain
    }

    pub(crate) fn baseline_scale(&self) -> f64 {
        self.baseline_scale
    }

    pub(crate) fn noise(&self) -> f64 {
        self.noise
    }

    pub(crate) fn impulse_scale(&self) -> f64 {
        self.impulse_scale
    }

    pub(crate) fn gating(&self) -> f64 {
        self.gating
    }

    pub(crate) fn family_share(&self) -> f64 {
        self.family_share
    }

    pub(crate) fn sensitivity_scale(&self) -> f64 {
        self.sensitivity_scale
    }

    pub(crate) fn bias_spread(&self) -> f64 {
        self.bias_spread
    }

    pub(crate) fn gain_plasticity(&self) -> f64 {
        self.gain_plasticity
    }

    pub(crate) fn coupling_plasticity(&self) -> f64 {
        self.coupling_plasticity
    }

    pub(crate) fn baseline_plasticity(&self) -> f64 {
        self.baseline_plasticity
    }

    pub(crate) fn habit_plasticity(&self) -> f64 {
        self.habit_plasticity
    }

    pub(crate) fn coupling_polarity(&self) -> f64 {
        self.coupling_polarity
    }

    /// Multiplies one scale, for controlled experimental variations.
    pub(crate) fn scaled(mut self, name: &str, factor: f64) -> Result<Self, String> {
        let scale = match name {
            "time_scale" => &mut self.time_scale,
            "time_spread" => &mut self.time_spread,
            "coupling_gain" => &mut self.coupling_gain,
            "baseline_scale" => &mut self.baseline_scale,
            "noise" => &mut self.noise,
            "impulse_scale" => &mut self.impulse_scale,
            "gating" => &mut self.gating,
            "sensitivity_scale" => &mut self.sensitivity_scale,
            "bias_spread" => &mut self.bias_spread,
            "gain_plasticity" => &mut self.gain_plasticity,
            "coupling_plasticity" => &mut self.coupling_plasticity,
            "baseline_plasticity" => &mut self.baseline_plasticity,
            "habit_plasticity" => &mut self.habit_plasticity,
            "coupling_polarity" => &mut self.coupling_polarity,
            other => return Err(format!("unknown constitution scale {other}")),
        };

        if !(factor.is_finite() && factor >= 0.0) {
            return Err(format!("invalid factor {factor}"));
        }

        *scale *= factor;
        Ok(self)
    }
}
