use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::influences::{
    influence::Influence, influence_evidence::InfluenceEvidence,
    influence_exposure::InfluenceExposure,
};

/// Influences of the unconscious on the conscious, and what became of each
/// of them in every episode it was exposed to, plus the replays that tested
/// whether it made a difference.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct InfluenceMemory {
    influences: Vec<Influence>,
    exposures: Vec<InfluenceExposure>,
    demonstrations: Vec<(Uuid, Uuid, bool)>,
}

impl InfluenceMemory {
    /// Bound on simultaneously active influences: a computational limit,
    /// so that influences cannot accumulate without review.
    pub(crate) const MAXIMUM_ACTIVE: usize = 64;

    pub(crate) fn influences(&self) -> &[Influence] {
        &self.influences
    }

    pub(crate) fn active(&self) -> impl Iterator<Item = &Influence> {
        self.influences
            .iter()
            .filter(|influence| influence.is_active())
    }

    pub(crate) fn exposures(&self) -> &[InfluenceExposure] {
        &self.exposures
    }

    /// Adds an influence unless an active one has the same content, or the
    /// bound is reached. Returns whether it was added.
    pub(crate) fn propose(&mut self, influence: Influence) -> bool {
        let known = self
            .active()
            .any(|active| active.kind() == influence.kind());

        if known || self.active().count() >= Self::MAXIMUM_ACTIVE {
            return false;
        }

        self.influences.push(influence);
        true
    }

    pub(crate) fn expose(&mut self, exposure: InfluenceExposure) {
        self.exposures.push(exposure);
    }

    pub(crate) fn demonstrate(&mut self, influence: Uuid, dream: Uuid, made_a_difference: bool) {
        self.demonstrations
            .push((influence, dream, made_a_difference));
    }

    /// Retires (or, with `None`, revives) an influence. Returns the previous
    /// retirement, to make the change reversible.
    pub(crate) fn set_retirement(
        &mut self,
        id: Uuid,
        retired: Option<(DateTime<Utc>, String)>,
    ) -> Result<Option<(DateTime<Utc>, String)>, String> {
        let influence = self
            .influences
            .iter_mut()
            .find(|influence| influence.id() == id)
            .ok_or_else(|| format!("unknown influence {id}"))?;
        let previous = influence.retirement().cloned();

        influence.set_retirement(retired);
        Ok(previous)
    }

    /// Evidence gathered in reality on each influence.
    pub(crate) fn evidence(&self) -> BTreeMap<Uuid, InfluenceEvidence> {
        let mut evidence: BTreeMap<Uuid, InfluenceEvidence> = BTreeMap::new();

        for exposure in self.exposures.iter().filter(|exposure| exposure.is_real()) {
            let entry = evidence.entry(exposure.influence()).or_default();

            entry.expose();

            for (_, outcome) in exposure.decisive() {
                entry.decide(outcome.is_success(true));
            }

            if exposure.supported_answer() {
                entry.support();
            }

            if exposure.refuted() {
                entry.refute();
            }
        }

        for (influence, _, made_a_difference) in &self.demonstrations {
            evidence
                .entry(*influence)
                .or_default()
                .demonstrate(*made_a_difference);
        }

        evidence
    }
}
