use serde::{Deserialize, Serialize};

use crate::memory::{invocation_outcome::InvocationOutcome, selection_record::SelectionRecord};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct InvocationRecord {
    step: usize,
    module: String,
    selection: SelectionRecord,
    duration_micros: u64,
    produced: Vec<usize>,
    produced_features: Vec<String>,
    rejected: Vec<String>,
    notes: Vec<String>,
    error: Option<String>,
    outcome: InvocationOutcome,
}

impl InvocationRecord {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        step: usize,
        module: String,
        selection: SelectionRecord,
        duration_micros: u64,
        produced: Vec<usize>,
        produced_features: Vec<String>,
        rejected: Vec<String>,
        notes: Vec<String>,
        error: Option<String>,
    ) -> Self {
        let outcome = if error.is_some() {
            InvocationOutcome::Failed
        } else {
            InvocationOutcome::Unproductive
        };

        Self {
            step,
            module,
            selection,
            duration_micros,
            produced,
            produced_features,
            rejected,
            notes,
            error,
            outcome,
        }
    }

    pub(crate) fn step(&self) -> usize {
        self.step
    }

    pub(crate) fn module(&self) -> &str {
        &self.module
    }

    pub(crate) fn selection(&self) -> &SelectionRecord {
        &self.selection
    }

    pub(crate) fn duration_micros(&self) -> u64 {
        self.duration_micros
    }

    /// Indexes of the reasoning-graph nodes created by this invocation.
    pub(crate) fn produced(&self) -> &[usize] {
        &self.produced
    }

    pub(crate) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub(crate) fn outcome(&self) -> InvocationOutcome {
        self.outcome
    }

    pub(crate) fn assess(&mut self, outcome: InvocationOutcome) {
        self.outcome = outcome;
    }
}
