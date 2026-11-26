use serde::{Deserialize, Serialize};

use crate::{
    consumables::resource_usage::ResourceUsage,
    planning::{plan::Plan, plan_status::PlanStatus},
};

/// Execution trace of an adopted plan: when it was adopted and left, which
/// steps were carried out or failed, what it actually cost compared with
/// what was anticipated, and whether its goals were reached.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct PlanRecord {
    plan: Plan,
    adopted_at: usize,
    ended_at: Option<usize>,
    status: PlanStatus,
    executed: Vec<(usize, usize)>,
    failed: Vec<(usize, usize)>,
    revision_reason: Option<String>,
    achieved: bool,
    actual_seconds: f64,
    actual_usage: ResourceUsage,
    actual_cost: f64,
}

impl PlanRecord {
    pub(crate) fn new(plan: Plan, adopted_at: usize) -> Self {
        Self {
            plan,
            adopted_at,
            ended_at: None,
            status: PlanStatus::Active,
            executed: Vec::new(),
            failed: Vec::new(),
            revision_reason: None,
            achieved: false,
            actual_seconds: 0.0,
            actual_usage: ResourceUsage::default(),
            actual_cost: 0.0,
        }
    }

    pub(crate) fn plan(&self) -> &Plan {
        &self.plan
    }

    pub(crate) fn adopted_at(&self) -> usize {
        self.adopted_at
    }

    pub(crate) fn status(&self) -> PlanStatus {
        self.status
    }

    pub(crate) fn is_active(&self) -> bool {
        self.status == PlanStatus::Active
    }

    pub(crate) fn achieved(&self) -> bool {
        self.achieved
    }

    pub(crate) fn actual_cost(&self) -> f64 {
        self.actual_cost
    }

    pub(crate) fn has_failures(&self) -> bool {
        !self.failed.is_empty()
    }

    fn is_done(&self, step: usize) -> bool {
        self.executed.iter().any(|(done, _)| *done == step)
    }

    /// Steps not carried out yet whose dependencies all are.
    pub(crate) fn ready_steps(&self) -> Vec<usize> {
        (0..self.plan.steps().len())
            .filter(|step| !self.is_done(*step))
            .filter(|step| {
                self.plan.steps()[*step]
                    .depends_on()
                    .iter()
                    .all(|dependency| self.is_done(*dependency))
            })
            .collect()
    }

    /// Steps of the plan carried out by this module, if any.
    pub(crate) fn steps_of(&self, module: &str) -> Vec<usize> {
        (0..self.plan.steps().len())
            .filter(|step| self.plan.steps()[*step].module() == module)
            .collect()
    }

    /// Accounts for one invocation made while the plan was active.
    pub(crate) fn account(
        &mut self,
        module: &str,
        invocation: usize,
        succeeded: bool,
        seconds: f64,
        usage: &ResourceUsage,
        cost: f64,
    ) {
        self.actual_seconds += seconds;
        self.actual_usage.add(usage);
        self.actual_cost += cost;

        let ready = self.ready_steps();

        for step in self.steps_of(module) {
            if !ready.contains(&step) {
                continue;
            }

            if succeeded {
                self.executed.push((step, invocation));
            } else {
                self.failed.push((step, invocation));
            }
        }

        if self.executed.len() == self.plan.steps().len() {
            self.status = PlanStatus::Completed;
            self.ended_at = Some(invocation);
        }
    }

    pub(crate) fn revise(&mut self, at: usize, reason: String) {
        self.status = PlanStatus::Revised;
        self.ended_at = Some(at);
        self.revision_reason = Some(reason);
    }

    /// Closes the record at the end of the episode.
    pub(crate) fn conclude(&mut self, at: usize, achieved: bool) {
        if self.status == PlanStatus::Active {
            self.status = PlanStatus::Abandoned;
            self.ended_at = Some(at);
        }

        self.achieved = achieved && self.status != PlanStatus::Revised;
    }
}
