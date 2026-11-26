use serde::{Deserialize, Serialize};

/// Observed effectiveness of one strategy for one kind of goals: how often
/// its plans reached their goals, and their actual versus predicted cost.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub(crate) struct StrategyStatistic {
    executions: u64,
    achieved: u64,
    total_cost: f64,
    total_predicted_cost: f64,
}

impl StrategyStatistic {
    pub(crate) fn observe(&mut self, achieved: bool, cost: f64, predicted_cost: f64) {
        self.executions += 1;
        self.total_cost += cost;
        self.total_predicted_cost += predicted_cost;

        if achieved {
            self.achieved += 1;
        }
    }

    pub(crate) fn executions(&self) -> u64 {
        self.executions
    }

    pub(crate) fn achieved(&self) -> u64 {
        self.achieved
    }

    pub(crate) fn mean_cost(&self) -> Option<f64> {
        (self.executions > 0).then(|| self.total_cost / self.executions as f64)
    }
}
