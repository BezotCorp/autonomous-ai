use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::world::{
    environment::Environment, format_one::transmission_v1::TransmissionV1, member::Member,
};

/// An environment as format 1 stored it. Read only: migrating keeps every
/// field, messages gaining an empty list of rules.
#[derive(Deserialize)]
pub(crate) struct EnvironmentV1 {
    id: Uuid,
    name: String,
    seed: u64,
    created_at: DateTime<Utc>,
    clock: f64,
    latency: f64,
    perceptibility: f64,
    external: bool,
    members: Vec<Member>,
    transmissions: Vec<TransmissionV1>,
}

impl EnvironmentV1 {
    pub(crate) fn migrate(self) -> Environment {
        Environment::restore(
            self.id,
            self.name,
            self.seed,
            self.created_at,
            self.clock,
            self.latency,
            self.perceptibility,
            self.external,
            self.members,
            self.transmissions
                .into_iter()
                .map(TransmissionV1::migrate)
                .collect(),
        )
    }
}
