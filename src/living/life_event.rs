use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    living::{
        activity_ending::ActivityEnding, activity_origin::ActivityOrigin, decision::Decision,
        vigilance::Vigilance,
    },
    memory::activity::Activity,
};

/// What the timeline records. Each entry is an observation of what the
/// mechanisms did; none is an interpretation.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LifeEvent {
    Born,
    SessionOpened {
        wall_gap: f64,
        dormancy_applied: f64,
    },
    SessionClosed,
    StimulusReceived {
        stimulus: Uuid,
        event: String,
        salience: f64,
    },
    StimulusNeglected {
        stimulus: Uuid,
        event: String,
    },
    ActivityStarted {
        activity: Uuid,
        kind: Activity,
        origin: ActivityOrigin,
        focus: String,
        decision: Option<Decision>,
    },
    ActivitySuspended {
        activity: Uuid,
        cause: String,
    },
    ActivityResumed {
        activity: Uuid,
        decision: Decision,
    },
    ActivityEnded {
        activity: Uuid,
        kind: Activity,
        ending: ActivityEnding,
        summary: String,
    },
    Slice {
        activity: Uuid,
        kind: Activity,
        lived: f64,
        summary: String,
    },
    Vigilance {
        to: Vigilance,
        decision: Decision,
    },
    ConcernFormed {
        concern: Uuid,
        subject: String,
        origin: String,
        salience: f64,
    },
    ConcernSurfaced {
        concern: Uuid,
        decision: Decision,
    },
    ConcernConcluded {
        concern: Uuid,
        how: String,
    },
    Occurrence {
        signature: String,
        magnitude: f64,
        impulse: f64,
    },
    Latent {
        state: Vec<f64>,
    },
    Response {
        event: String,
        answers: usize,
    },
    Intervention {
        description: String,
    },
    TechnicalError {
        detail: String,
    },
    /// The existence emitted a message in its environment.
    Transmitted {
        transmission: uuid::Uuid,
        to: String,
        status: String,
    },
    /// Channel values and plastic changes at a moment, so that their
    /// evolution over any period can be reported.
    Plasticity {
        channels: Vec<f64>,
        coupling_change: f64,
        baseline_change: f64,
        engagements: u64,
    },
    /// The existence became present in an environment, under a handle.
    Present {
        environment: String,
        handle: String,
    },
}

impl LifeEvent {
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::Born => "born",
            Self::SessionOpened { .. } => "session_opened",
            Self::SessionClosed => "session_closed",
            Self::StimulusReceived { .. } => "stimulus_received",
            Self::StimulusNeglected { .. } => "stimulus_neglected",
            Self::ActivityStarted { .. } => "activity_started",
            Self::ActivitySuspended { .. } => "activity_suspended",
            Self::ActivityResumed { .. } => "activity_resumed",
            Self::ActivityEnded { .. } => "activity_ended",
            Self::Slice { .. } => "slice",
            Self::Vigilance { .. } => "vigilance",
            Self::ConcernFormed { .. } => "concern_formed",
            Self::ConcernSurfaced { .. } => "concern_surfaced",
            Self::ConcernConcluded { .. } => "concern_concluded",
            Self::Occurrence { .. } => "occurrence",
            Self::Latent { .. } => "latent",
            Self::Response { .. } => "response",
            Self::Intervention { .. } => "intervention",
            Self::TechnicalError { .. } => "technical_error",
            Self::Transmitted { .. } => "transmitted",
            Self::Plasticity { .. } => "plasticity",
            Self::Present { .. } => "present",
        }
    }
}
