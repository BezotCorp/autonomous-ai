use uuid::Uuid;

use crate::{
    living::{activity_focus::ActivityFocus, activity_origin::ActivityOrigin},
    memory::activity::Activity,
};

/// A change of the life that may happen at some moment, at a rate.
#[derive(Clone, Debug)]
pub(crate) enum Transition {
    /// Act on a waiting stimulus.
    Attend(Uuid),
    /// Begin an activity.
    Start(Activity, Box<ActivityFocus>, ActivityOrigin),
    /// Resume a suspended activity.
    Resume(Uuid),
    FallAsleep,
    Wake,
    /// An unconscious concern becomes conscious.
    Surface(Uuid),
}
