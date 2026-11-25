use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{event::Event, observation::Observation};

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Memory {
    events: BTreeMap<String, Event>,
    observations: Vec<Observation>,
}

impl Memory {
    pub(crate) fn event_count(&self) -> usize {
        self.events.len()
    }

    pub(crate) fn observation_count(&self) -> usize {
        self.observations.len()
    }

    pub(crate) fn failure_count(&self) -> usize {
        self.observations
            .iter()
            .filter(|observation| observation.is_failure())
            .count()
    }

    pub(crate) fn contains_event(&self, id: &str) -> bool {
        self.events.contains_key(id)
    }

    pub(crate) fn record_event(&mut self, event: Event) {
        self.events.insert(event.id().to_owned(), event);
    }

    pub(crate) fn record_observation(&mut self, observation: Observation) {
        self.observations.push(observation);
    }
}
