use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::memory::{correction_record::CorrectionRecord, invocation_outcome::InvocationOutcome};

/// Every correction the examination of the cognition applied, in order,
/// with the outcomes of past invocations it re-examined (which the routing
/// memory uses instead of the outcomes assessed at the time).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct CorrectionJournal {
    records: Vec<CorrectionRecord>,
    reassessments: BTreeMap<(Uuid, usize), InvocationOutcome>,
}

impl CorrectionJournal {
    pub(crate) fn records(&self) -> &[CorrectionRecord] {
        &self.records
    }

    pub(crate) fn record(&mut self, record: CorrectionRecord) {
        self.records.push(record);
    }

    pub(crate) fn record_mut(&mut self, id: Uuid) -> Option<&mut CorrectionRecord> {
        self.records.iter_mut().find(|record| record.id() == id)
    }

    pub(crate) fn reassessments(&self) -> &BTreeMap<(Uuid, usize), InvocationOutcome> {
        &self.reassessments
    }

    pub(crate) fn reassess(
        &mut self,
        episode: Uuid,
        step: usize,
        outcome: Option<InvocationOutcome>,
    ) -> Option<InvocationOutcome> {
        match outcome {
            Some(outcome) => self.reassessments.insert((episode, step), outcome),
            None => self.reassessments.remove(&(episode, step)),
        }
    }

    /// Whether an active correction already concerns this subject.
    pub(crate) fn covers(&self, subject: &str) -> bool {
        self.records
            .iter()
            .any(|record| record.is_active() && record.correction().subject() == subject)
    }
}
