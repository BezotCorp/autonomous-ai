use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    living::{activity_focus::ActivityFocus, activity_origin::ActivityOrigin},
    memory::activity::Activity,
};

/// An activity under way or suspended. It advances by slices (an episode
/// of the engine, or an unfolding of the imagination); between slices it
/// may be interrupted, suspended, resumed or transformed.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct OngoingActivity {
    id: Uuid,
    activity: Activity,
    origin: ActivityOrigin,
    focus: ActivityFocus,
    started_at: f64,
    lived: f64,
    slices: u32,
    suspended: bool,
    hold: f64,
    episodes: Vec<Uuid>,
}

impl OngoingActivity {
    pub(crate) fn new(
        activity: Activity,
        origin: ActivityOrigin,
        focus: ActivityFocus,
        started_at: f64,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            activity,
            origin,
            focus,
            started_at,
            lived: 0.0,
            slices: 0,
            suspended: false,
            hold: 1.0,
            episodes: Vec::new(),
        }
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn activity(&self) -> Activity {
        self.activity
    }

    pub(crate) fn origin(&self) -> &ActivityOrigin {
        &self.origin
    }

    pub(crate) fn focus(&self) -> &ActivityFocus {
        &self.focus
    }

    pub(crate) fn started_at(&self) -> f64 {
        self.started_at
    }

    pub(crate) fn lived(&self) -> f64 {
        self.lived
    }

    pub(crate) fn slices(&self) -> u32 {
        self.slices
    }

    pub(crate) fn is_suspended(&self) -> bool {
        self.suspended
    }

    /// How strongly a suspended activity is still held (fades with time).
    pub(crate) fn hold(&self) -> f64 {
        self.hold
    }

    pub(crate) fn account(&mut self, lived: f64, episode: Option<Uuid>) {
        if lived.is_finite() && lived > 0.0 {
            self.lived += lived;
        }

        self.slices += 1;
        self.episodes.extend(episode);
    }

    pub(crate) fn suspend(&mut self) {
        self.suspended = true;
        self.hold = 1.0;
    }

    pub(crate) fn resume(&mut self) {
        self.suspended = false;
    }

    pub(crate) fn fade_hold(&mut self, factor: f64) {
        if factor.is_finite() {
            self.hold = (self.hold * factor).max(0.0);
        }
    }
}
