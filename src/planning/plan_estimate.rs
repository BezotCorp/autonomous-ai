use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Anticipated outcome of a plan before its execution: probability of
/// reaching its goals, duration, consumption of each resource, and the
/// resulting utility (success minus time and scarcity costs).
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct PlanEstimate {
    success: f64,
    seconds: f64,
    consumption: BTreeMap<String, f64>,
    scarcity: f64,
    cost: f64,
    utility: f64,
}

impl PlanEstimate {
    pub(crate) fn new(
        success: f64,
        seconds: f64,
        consumption: BTreeMap<String, f64>,
        scarcity: f64,
        cost: f64,
    ) -> Self {
        Self {
            success,
            seconds,
            consumption,
            scarcity,
            cost,
            utility: success - cost,
        }
    }

    pub(crate) fn cost(&self) -> f64 {
        self.cost
    }

    pub(crate) fn utility(&self) -> f64 {
        self.utility
    }
}
