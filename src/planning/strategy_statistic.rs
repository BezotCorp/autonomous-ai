use serde::{Deserialize, Serialize};

/// Observed effectiveness of one strategy for one kind of goals: how often
/// its plans reached their goals, and their actual versus predicted cost.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub(crate) struct StrategyStatistic {
    executions: u64,
    achieved: u64,
    total_cost: f64,
    total_predicted_cost: f64,
    simulated_executions: u64,
    simulated_achieved: u64,
    simulated_cost: f64,
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

    /// Records a plan executed in a dream, kept apart from real executions.
    pub(crate) fn observe_simulated(&mut self, achieved: bool, cost: f64) {
        self.simulated_executions += 1;
        self.simulated_cost += cost;

        if achieved {
            self.simulated_achieved += 1;
        }
    }

    pub(crate) fn simulated_executions(&self) -> u64 {
        self.simulated_executions
    }

    pub(crate) fn simulated_achieved(&self) -> u64 {
        self.simulated_achieved
    }

    pub(crate) fn mean_simulated_cost(&self) -> Option<f64> {
        (self.simulated_executions > 0)
            .then(|| self.simulated_cost / self.simulated_executions as f64)
    }

    pub(crate) fn mean_predicted_cost(&self) -> Option<f64> {
        (self.executions > 0).then(|| self.total_predicted_cost / self.executions as f64)
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
