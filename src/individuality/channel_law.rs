use serde::{Deserialize, Serialize};

use crate::individuality::channel_form::ChannelForm;

/// Individual law of a channel: its activation is `bias + sensitivity · z`,
/// where `z` is the latent state, then shaped by the form of the channel.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct ChannelLaw {
    bias: f64,
    sensitivity: Vec<f64>,
}

impl ChannelLaw {
    /// Bound on any activation, so that every derived value stays finite
    /// (a rate below e^25 per hour, a probability strictly within (0, 1)).
    pub(crate) const MAXIMUM_ACTIVATION: f64 = 25.0;

    pub(crate) fn new(bias: f64, sensitivity: Vec<f64>) -> Self {
        Self { bias, sensitivity }
    }

    pub(crate) fn bias(&self) -> f64 {
        self.bias
    }

    pub(crate) fn sensitivity(&self) -> &[f64] {
        &self.sensitivity
    }

    /// Contribution of the latent state to the activation.
    pub(crate) fn latent_term(&self, latent: &[f64], plastic: Option<&[f64]>) -> f64 {
        self.sensitivity
            .iter()
            .enumerate()
            .map(|(index, weight)| {
                (weight
                    + plastic
                        .and_then(|plastic| plastic.get(index))
                        .unwrap_or(&0.0))
                    * latent.get(index).copied().unwrap_or(0.0)
            })
            .sum()
    }

    pub(crate) fn shape(form: ChannelForm, activation: f64) -> f64 {
        let activation = if activation.is_nan() {
            0.0
        } else {
            activation.clamp(-Self::MAXIMUM_ACTIVATION, Self::MAXIMUM_ACTIVATION)
        };

        match form {
            ChannelForm::Rate | ChannelForm::Factor => activation.exp(),
            ChannelForm::Probability => 1.0 / (1.0 + (-activation).exp()),
        }
    }
}
