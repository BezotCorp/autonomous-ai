use serde::{Deserialize, Serialize};

/// How a channel turns its activation (bias plus the projection of the
/// latent state) into the value a mechanism uses.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ChannelForm {
    /// A rate per hour: `exp(activation)`.
    Rate,
    /// A positive factor around 1: `exp(activation)`.
    Factor,
    /// A probability: `1 / (1 + exp(-activation))`.
    Probability,
}
