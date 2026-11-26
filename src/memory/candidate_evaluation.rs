use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// How the selector valued one candidate module.
///
/// - `stake`: what remains to gain from acting — 1 when something is
///   unknown, the probability that a remembered answer is outdated when
///   the module would only verify it;
/// - success: posterior mean and variance learned from experience, and the
///   exploratory sample actually drawn;
/// - costs: expected duration and consumption of each resource, and the
///   scarcity cost of that consumption given what remains of it;
/// - `utility`: stake × sampled success − time and scarcity costs.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct CandidateEvaluation {
    stake: f64,
    expected_success: f64,
    success_variance: f64,
    sampled_success: f64,
    expected_seconds: f64,
    expected_consumption: BTreeMap<String, f64>,
    scarcity_cost: f64,
    cost: f64,
    expected_utility: f64,
    utility: f64,
}

impl CandidateEvaluation {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        stake: f64,
        expected_success: f64,
        success_variance: f64,
        sampled_success: f64,
        expected_seconds: f64,
        expected_consumption: BTreeMap<String, f64>,
        scarcity_cost: f64,
        cost: f64,
    ) -> Self {
        Self {
            stake,
            expected_success,
            success_variance,
            sampled_success,
            expected_seconds,
            expected_consumption,
            scarcity_cost,
            cost,
            expected_utility: stake * expected_success - cost,
            utility: stake * sampled_success - cost,
        }
    }

    /// Utility with the posterior mean instead of the exploratory sample.
    pub(crate) fn expected_utility(&self) -> f64 {
        self.expected_utility
    }

    pub(crate) fn utility(&self) -> f64 {
        self.utility
    }

    /// Expected cost in utility units (time and scarcity).
    pub(crate) fn cost(&self) -> f64 {
        self.cost
    }

    pub(crate) fn is_feasible(&self) -> bool {
        self.scarcity_cost.is_finite()
    }
}
