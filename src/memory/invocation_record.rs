use serde::{Deserialize, Serialize};

use crate::{
    consumables::resource_usage::ResourceUsage,
    memory::{invocation_outcome::InvocationOutcome, selection_record::SelectionRecord},
};

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
    usage: ResourceUsage,
    actual_cost: f64,
    adopted_plan: bool,
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
            usage: ResourceUsage::default(),
            actual_cost: 0.0,
            adopted_plan: false,
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

    /// Records the resources consumed and the resulting cost in utility
    /// units (time and scarcity).
    pub(crate) fn account(&mut self, usage: ResourceUsage, actual_cost: f64) {
        self.usage = usage;
        self.actual_cost = actual_cost;
    }

    pub(crate) fn reject(&mut self, reason: String) {
        self.rejected.push(reason);
    }

    pub(crate) fn adopt_plan(&mut self) {
        self.adopted_plan = true;
    }

    pub(crate) fn adopted_plan(&self) -> bool {
        self.adopted_plan
    }

    pub(crate) fn usage(&self) -> &ResourceUsage {
        &self.usage
    }

    pub(crate) fn actual_cost(&self) -> f64 {
        self.actual_cost
    }

    pub(crate) fn produced_features(&self) -> &[String] {
        &self.produced_features
    }

    pub(crate) fn assess(&mut self, outcome: InvocationOutcome) {
        self.outcome = outcome;
    }
}
