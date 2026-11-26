use serde::Serialize;

use crate::memory::invocation_outcome::InvocationOutcome;

/// Usage and outcomes of one module over every recorded episode.
#[derive(Debug, Default, Serialize)]
pub(crate) struct ModuleStatistics {
    invocations: u64,
    contributed: u64,
    advanced: u64,
    retained: u64,
    invalidated: u64,
    unproductive: u64,
    failed: u64,
    mean_duration_seconds: f64,
}

impl ModuleStatistics {
    pub(crate) fn observe(&mut self, outcome: InvocationOutcome, micros: u64) {
        let total = self.mean_duration_seconds * self.invocations as f64 + micros as f64 / 1e6;

        self.invocations += 1;
        self.mean_duration_seconds = total / self.invocations as f64;

        match outcome {
            InvocationOutcome::Contributed => self.contributed += 1,
            InvocationOutcome::Advanced => self.advanced += 1,
            InvocationOutcome::Retained => self.retained += 1,
            InvocationOutcome::Invalidated => self.invalidated += 1,
            InvocationOutcome::Unproductive => self.unproductive += 1,
            InvocationOutcome::Failed => self.failed += 1,
        }
    }
}
