use serde::{Deserialize, Serialize};

use crate::individuality::channel::Channel;

/// Decomposition of the rate (per hour) at which a transition could happen
/// at a given moment: `rate = exp(bias + latent + context)`. The latent
/// term is the exact contribution of the internal state to the logarithm
/// of the rate: setting the state to zero would multiply the rate by
/// `exp(−latent)`, everything else being equal. This is a causal
/// statement about the mechanism, not about behaviour, which also depends
/// on chance and on everything else that competed at that moment.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Decision {
    channel: Channel,
    bias: f64,
    latent: f64,
    context: f64,
    rate: f64,
}

impl Decision {
    /// Bound on any rate of transition (per hour): once a second.
    pub(crate) const MAXIMUM_RATE: f64 = 3_600.0;

    pub(crate) fn new(channel: Channel, bias: f64, latent: f64, context: f64) -> Self {
        let activation = bias + latent + context;
        let rate = if activation.is_nan() {
            0.0
        } else {
            activation.clamp(-50.0, 50.0).exp().min(Self::MAXIMUM_RATE)
        };

        Self {
            channel,
            bias,
            latent,
            context,
            rate,
        }
    }

    pub(crate) fn rate(&self) -> f64 {
        self.rate
    }
}
