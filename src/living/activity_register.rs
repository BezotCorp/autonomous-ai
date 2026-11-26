use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::living::{ended_activity::EndedActivity, ongoing_activity::OngoingActivity};

/// Activities under way (running or suspended) and finished ones.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct ActivityRegister {
    ongoing: Vec<OngoingActivity>,
    ended: Vec<EndedActivity>,
    dropped: u64,
}

impl ActivityRegister {
    /// Bound on the finished activities kept in detail.
    const MAXIMUM_ENDED: usize = 20_000;

    pub(crate) fn ended(&self) -> &[EndedActivity] {
        &self.ended
    }

    pub(crate) fn dropped(&self) -> u64 {
        self.dropped
    }

    pub(crate) fn running(&self) -> impl Iterator<Item = &OngoingActivity> {
        self.ongoing
            .iter()
            .filter(|activity| !activity.is_suspended())
    }

    pub(crate) fn suspended(&self) -> impl Iterator<Item = &OngoingActivity> {
        self.ongoing
            .iter()
            .filter(|activity| activity.is_suspended())
    }

    pub(crate) fn get(&self, id: Uuid) -> Option<&OngoingActivity> {
        self.ongoing.iter().find(|activity| activity.id() == id)
    }

    pub(crate) fn get_mut(&mut self, id: Uuid) -> Option<&mut OngoingActivity> {
        self.ongoing.iter_mut().find(|activity| activity.id() == id)
    }

    pub(crate) fn start(&mut self, activity: OngoingActivity) {
        self.ongoing.push(activity);
    }

    pub(crate) fn remove(&mut self, id: Uuid) -> Option<OngoingActivity> {
        let position = self
            .ongoing
            .iter()
            .position(|activity| activity.id() == id)?;

        Some(self.ongoing.remove(position))
    }

    pub(crate) fn finish(&mut self, ended: EndedActivity) {
        self.ended.push(ended);

        if self.ended.len() > Self::MAXIMUM_ENDED {
            self.ended.remove(0);
            self.dropped += 1;
        }
    }

    pub(crate) fn ongoing_mut(&mut self) -> impl Iterator<Item = &mut OngoingActivity> {
        self.ongoing.iter_mut()
    }
}
