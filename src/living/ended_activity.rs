use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    living::{activity_ending::ActivityEnding, activity_origin::ActivityOrigin},
    memory::activity::Activity,
};

/// Summary of a finished activity, kept for observation and introspection.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct EndedActivity {
    id: Uuid,
    activity: Activity,
    origin: ActivityOrigin,
    focus: String,
    started_at: f64,
    ended_at: f64,
    lived: f64,
    slices: u32,
    ending: ActivityEnding,
    summary: String,
}

impl EndedActivity {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: Uuid,
        activity: Activity,
        origin: ActivityOrigin,
        focus: String,
        started_at: f64,
        ended_at: f64,
        lived: f64,
        slices: u32,
        ending: ActivityEnding,
        summary: String,
    ) -> Self {
        Self {
            id,
            activity,
            origin,
            focus,
            started_at,
            ended_at,
            lived,
            slices,
            ending,
            summary,
        }
    }

    pub(crate) fn activity(&self) -> Activity {
        self.activity
    }

    pub(crate) fn ending(&self) -> ActivityEnding {
        self.ending
    }

    pub(crate) fn lived(&self) -> f64 {
        self.lived
    }

    pub(crate) fn slices(&self) -> u32 {
        self.slices
    }
}
