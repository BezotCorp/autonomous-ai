use std::collections::BTreeMap;

use serde::Serialize;

use crate::memory::{dream_record::DreamRecord, dream_verdict::DreamVerdict};

/// What dreams found, by mode (historical replays and reinterpretations
/// are never mixed): verdicts, and the simulated gains of those that found
/// a cheaper way to the same answers.
#[derive(Debug, Default, Serialize)]
pub(crate) struct DreamStatistics {
    dreams: usize,
    verdicts: BTreeMap<String, usize>,
    simulated_invocations_saved: i64,
    simulated_consumption_saved: f64,
    hypotheses_discovered: usize,
}

impl DreamStatistics {
    pub(crate) fn observe(&mut self, dream: &DreamRecord) {
        self.dreams += 1;
        *self
            .verdicts
            .entry(format!("{:?}", dream.verdict()))
            .or_default() += 1;
        self.hypotheses_discovered += dream.hypotheses().len();

        if dream.verdict() == DreamVerdict::Cheaper {
            self.simulated_invocations_saved += dream.original_outcome().invocations() as i64
                - dream.dream_outcome().invocations() as i64;
            self.simulated_consumption_saved += dream.original_outcome().total_consumption()
                - dream.dream_outcome().total_consumption();
        }
    }
}
