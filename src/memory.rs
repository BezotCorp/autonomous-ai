use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    cargo_snapshot::CargoSnapshot, event::Event, knowledge::Knowledge, observation::Observation,
    observation_payload::ObservationPayload, publication_analysis::PublicationAnalysis,
    snapshot_identity::SnapshotIdentity,
};

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Memory {
    events: BTreeMap<String, Event>,
    observations: Vec<Observation>,
    knowledge: Knowledge,
    publication_analyses: Vec<PublicationAnalysis>,
}

impl Memory {
    pub(crate) fn knowledge(&self) -> &Knowledge {
        &self.knowledge
    }

    pub(crate) fn integrate_snapshot(
        &mut self,
        identity: SnapshotIdentity,
        snapshot: CargoSnapshot,
    ) {
        self.knowledge.integrate(identity, snapshot);
    }

    pub(crate) fn record_publication_analysis(&mut self, analysis: PublicationAnalysis) {
        self.publication_analyses.push(analysis);
    }

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

    pub(crate) fn record_observation(&mut self, event_id: &str, payload: ObservationPayload) {
        self.observations
            .push(Observation::new(event_id.to_owned(), payload));
    }
}
