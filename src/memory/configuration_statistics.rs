use serde::Serialize;

use crate::{
    consumables::resource_usage::ResourceUsage,
    memory::{
        answer_status::AnswerStatus, episode_record::EpisodeRecord, termination::Termination,
    },
    planning::plan_status::PlanStatus,
};

/// Aggregated results of the episodes run under one experimental
/// configuration: quality and currency of the answers, effort, external
/// requests, plans and use of the persistent knowledge.
#[derive(Debug, Default, Serialize)]
pub(crate) struct ConfigurationStatistics {
    episodes: usize,
    invocations: usize,
    proven_or_refuted: usize,
    undetermined: usize,
    current_answers: usize,
    historical_answers: usize,
    mean_answer_staleness: f64,
    revisions: usize,
    total_seconds: f64,
    predicted_invocation_cost: f64,
    actual_invocation_cost: f64,
    usage: ResourceUsage,
    resources_exhausted: usize,
    infeasible_selections: usize,
    failed_invocations: usize,
    plans_adopted: usize,
    plans_revised: usize,
    plans_completed: usize,
    plans_achieved: usize,
    plans_reused: usize,
    mean_plan_cost_error: f64,
    recalled: usize,
    recalled_in_answers: usize,
    recollections_confirmed: usize,
    recollections_superseded: usize,
}

impl ConfigurationStatistics {
    pub(crate) fn label(episode: &EpisodeRecord) -> String {
        episode.configuration().label()
    }

    pub(crate) fn observe(&mut self, episode: &EpisodeRecord) {
        let answered = (self.proven_or_refuted + self.undetermined) as f64;
        let mut staleness = self.mean_answer_staleness * answered;
        let plans = self.plans_adopted as f64;
        let mut cost_error = self.mean_plan_cost_error * plans;

        self.episodes += 1;
        self.invocations += episode.invocations().len();
        self.revisions += episode.revisions().len();
        self.total_seconds += episode.duration_micros() as f64 / 1e6;
        self.usage.add(episode.usage());

        if episode.termination() == Termination::ResourcesExhausted {
            self.resources_exhausted += 1;
        }

        for invocation in episode.invocations() {
            self.infeasible_selections += invocation.selection().infeasible();

            if invocation.selection().evaluation().is_feasible() {
                self.predicted_invocation_cost += invocation.selection().evaluation().cost();
            }

            self.actual_invocation_cost += invocation.actual_cost();

            if invocation.error().is_some() {
                self.failed_invocations += 1;
            }
        }

        for answer in episode.answers() {
            match answer.status() {
                AnswerStatus::Proven | AnswerStatus::Refuted => {
                    self.proven_or_refuted += 1;

                    if answer.is_current() {
                        self.current_answers += 1;
                    } else {
                        self.historical_answers += 1;
                    }
                }
                AnswerStatus::Undetermined | AnswerStatus::Contradictory => self.undetermined += 1,
            }

            staleness += answer.staleness();
        }

        for record in episode.plans() {
            self.plans_adopted += 1;
            cost_error += (record.actual_cost() - record.plan().estimate().cost()).abs();

            match record.status() {
                PlanStatus::Revised => self.plans_revised += 1,
                PlanStatus::Completed => self.plans_completed += 1,
                PlanStatus::Active | PlanStatus::Abandoned => {}
            }

            if record.achieved() {
                self.plans_achieved += 1;
            }

            if record.plan().is_reused() {
                self.plans_reused += 1;
            }
        }

        let exploitation = episode.exploitation();

        self.recalled += exploitation.recall_count();
        self.recalled_in_answers += exploitation.recalled_in_answers();
        self.recollections_confirmed += exploitation.confirmed_count();
        self.recollections_superseded += exploitation.superseded_count();

        let answered = (self.proven_or_refuted + self.undetermined) as f64;

        if answered > 0.0 {
            self.mean_answer_staleness = staleness / answered;
        }

        if self.plans_adopted > 0 {
            self.mean_plan_cost_error = cost_error / self.plans_adopted as f64;
        }
    }
}
