use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    planning::{plan_estimate::PlanEstimate, plan_step::PlanStep},
    representation::statement::Statement,
};

/// Anticipated course of action towards some goals: which module should
/// supply which information, in which order, at which expected cost, and
/// how it compared with the alternative strategies considered. A plan
/// does not command the engine: it informs the choice of the next module,
/// whose adherence to plans is itself learned.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Plan {
    goals: Vec<Statement>,
    strategy: String,
    steps: Vec<PlanStep>,
    estimate: PlanEstimate,
    alternatives: Vec<(String, f64)>,
    reused: bool,
}

impl Plan {
    pub(crate) fn new(
        goals: Vec<Statement>,
        strategy: String,
        steps: Vec<PlanStep>,
        estimate: PlanEstimate,
        alternatives: Vec<(String, f64)>,
        reused: bool,
    ) -> Self {
        Self {
            goals,
            strategy,
            steps,
            estimate,
            alternatives,
            reused,
        }
    }

    /// Identifies the kind of problem the plan addresses, so that the
    /// strategies used for the same kind of goals can be compared.
    pub(crate) fn goal_key(goals: &[Statement]) -> String {
        goals
            .iter()
            .map(Statement::signature)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join("+")
    }

    pub(crate) fn goals(&self) -> &[Statement] {
        &self.goals
    }

    pub(crate) fn strategy(&self) -> &str {
        &self.strategy
    }

    pub(crate) fn steps(&self) -> &[PlanStep] {
        &self.steps
    }

    pub(crate) fn estimate(&self) -> &PlanEstimate {
        &self.estimate
    }

    pub(crate) fn is_reused(&self) -> bool {
        self.reused
    }
}
