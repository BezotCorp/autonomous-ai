use std::fmt;

use serde::{Deserialize, Serialize};

use crate::individuality::channel_form::ChannelForm;

/// A parameter of a cognitive mechanism that the latent state modulates.
///
/// Channels name mechanisms of the architecture (a rate of transition, a
/// sampling temperature, a probability of admitting an influence), never
/// psychological qualities. Which directions of the latent space act on a
/// channel, and how strongly, is drawn at birth for each individual: no
/// dimension of the latent is tied to a channel by the architecture.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Channel {
    /// Rate at which a received stimulus of unit salience gives rise to an
    /// action (per hour).
    Attend,
    /// Rate of spontaneous unconscious examination of the cognition.
    Reflect,
    /// Rate of spontaneous conscious examination of oneself.
    Introspect,
    /// Rate of directed mental simulation, per unit of salience of the
    /// conscious concerns.
    Simulate,
    /// Rate of free imagination while awake.
    Reverie,
    /// Rate of free imagination while asleep.
    Dream,
    /// Rate of passing from wakefulness to sleep.
    FallAsleep,
    /// Rate of passing from sleep to wakefulness (stimuli add to it).
    Wake,
    /// Rate of resuming a suspended activity, per unit of salience of what
    /// it was about.
    Resume,
    /// Hazard of interrupting a running activity (per hour of activity).
    Interrupt,
    /// Hazard of a running activity drifting into another kind.
    Transform,
    /// Temperature applied to the posteriors sampled by the selector.
    SamplingTemperature,
    /// Factor applied to the expected costs weighed by the selector.
    CostSensitivity,
    /// Probability that an applicable influence is admitted in a decision.
    InfluenceAdmission,
    /// Hazard of abandoning the reasoning, per invocation.
    Abandonment,
    /// Factor applied to the number of invocations of one slice of activity.
    SliceLength,
    /// Factor applied to the salience of received stimuli.
    StimulusGain,
    /// Rate at which the salience of a concern fades (per hour).
    ConcernFading,
    /// Rate at which an unconscious concern becomes conscious, per unit of
    /// salience.
    Surfacing,
    /// Factor applied to the salience a concern receives when it forms or
    /// is reactivated.
    ConcernGain,
    /// Probability that imagination goes on to another scene.
    SceneContinuation,
    /// Relative weights of the mechanisms of imagination.
    Recollection,
    Association,
    Transformation,
    Invention,
    ImaginedCapability,
    Anticipation,
    Constraint,
    SelfRepresentation,
}

impl Channel {
    pub(crate) const ALL: [Self; 29] = [
        Self::Attend,
        Self::Reflect,
        Self::Introspect,
        Self::Simulate,
        Self::Reverie,
        Self::Dream,
        Self::FallAsleep,
        Self::Wake,
        Self::Resume,
        Self::Interrupt,
        Self::Transform,
        Self::SamplingTemperature,
        Self::CostSensitivity,
        Self::InfluenceAdmission,
        Self::Abandonment,
        Self::SliceLength,
        Self::StimulusGain,
        Self::ConcernFading,
        Self::Surfacing,
        Self::ConcernGain,
        Self::SceneContinuation,
        Self::Recollection,
        Self::Association,
        Self::Transformation,
        Self::Invention,
        Self::ImaginedCapability,
        Self::Anticipation,
        Self::Constraint,
        Self::SelfRepresentation,
    ];

    /// Mechanisms of imagination, in the order the imagination knows them.
    pub(crate) const IMAGINATION: [Self; 8] = [
        Self::Recollection,
        Self::Association,
        Self::Transformation,
        Self::Invention,
        Self::ImaginedCapability,
        Self::Anticipation,
        Self::Constraint,
        Self::SelfRepresentation,
    ];

    pub(crate) fn form(self) -> ChannelForm {
        match self {
            Self::Attend
            | Self::Reflect
            | Self::Introspect
            | Self::Simulate
            | Self::Reverie
            | Self::Dream
            | Self::FallAsleep
            | Self::Wake
            | Self::Resume
            | Self::Interrupt
            | Self::Transform
            | Self::ConcernFading
            | Self::Surfacing => ChannelForm::Rate,
            Self::InfluenceAdmission | Self::SceneContinuation | Self::Abandonment => {
                ChannelForm::Probability
            }
            _ => ChannelForm::Factor,
        }
    }

    /// Centre of the distribution of the bias at birth, in activation
    /// units (log of a rate per hour, log of a factor, or log-odds). These
    /// are orders of magnitude chosen so that every mechanism can occur
    /// within hours of existence; they are not measured constants.
    pub(crate) fn centre(self) -> f64 {
        match self {
            Self::Attend => 30.0_f64.ln(),
            Self::Reflect => 1.5_f64.ln(),
            Self::Introspect => 1.0_f64.ln(),
            Self::Simulate => 1.0_f64.ln(),
            Self::Reverie => 2.0_f64.ln(),
            Self::Dream => 4.0_f64.ln(),
            Self::FallAsleep => (1.0_f64 / 8.0).ln(),
            Self::Wake => 0.5_f64.ln(),
            Self::Resume => 1.0_f64.ln(),
            Self::Interrupt => 0.5_f64.ln(),
            Self::Transform => 0.3_f64.ln(),
            Self::ConcernFading => 0.2_f64.ln(),
            Self::Surfacing => 1.0_f64.ln(),
            // About one chance in a hundred per invocation.
            Self::Abandonment => -4.6,
            // Three chances in four.
            Self::InfluenceAdmission => 1.1,
            // Seven chances in ten.
            Self::SceneContinuation => 0.85,
            _ => 0.0,
        }
    }

    pub(crate) fn label(self) -> String {
        format!("{self:?}")
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}
