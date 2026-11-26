use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::engine::learning_parameters::LearningParameters;

/// Conditions under which an episode ran, needed to compare experiments.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct EpisodeConfiguration {
    seed: u64,
    acquired_knowledge: bool,
    learned_routing: bool,
    cost_weight_per_second: f64,
    scarcity_weight: f64,
    minimum_utility: f64,
    resource_budgets: BTreeMap<String, f64>,
}

impl EpisodeConfiguration {
    pub(crate) fn new(seed: u64, acquired_knowledge: bool, learned_routing: bool) -> Self {
        Self {
            seed,
            acquired_knowledge,
            learned_routing,
            cost_weight_per_second: 0.0,
            scarcity_weight: 0.0,
            minimum_utility: 0.0,
            resource_budgets: BTreeMap::new(),
        }
    }

    /// Records the decision parameters the episode ran with.
    pub(crate) fn with_parameters(mut self, parameters: &LearningParameters) -> Self {
        self.cost_weight_per_second = parameters.cost_weight_per_second();
        self.scarcity_weight = parameters.scarcity_weight();
        self.minimum_utility = parameters.minimum_utility();
        self
    }

    /// Limits imposed on resources for the episode, on top of their own
    /// constraints (to study scarcity without exhausting real quotas).
    pub(crate) fn with_resource_budgets(mut self, budgets: BTreeMap<String, f64>) -> Self {
        self.resource_budgets = budgets;
        self
    }

    pub(crate) fn resource_budgets(&self) -> &BTreeMap<String, f64> {
        &self.resource_budgets
    }

    /// Label grouping comparable episodes.
    pub(crate) fn label(&self) -> String {
        let budgets: Vec<String> = self
            .resource_budgets
            .iter()
            .map(|(resource, units)| format!("{resource}={units}"))
            .collect();

        format!(
            "knowledge={} routing={} cost_weight={} scarcity_weight={} minimum_utility={}{}",
            self.acquired_knowledge,
            self.learned_routing,
            self.cost_weight_per_second,
            self.scarcity_weight,
            self.minimum_utility,
            if budgets.is_empty() {
                String::new()
            } else {
                format!(" budgets={}", budgets.join(","))
            }
        )
    }

    pub(crate) fn seed(&self) -> u64 {
        self.seed
    }

    pub(crate) fn acquired_knowledge(&self) -> bool {
        self.acquired_knowledge
    }

    pub(crate) fn learned_routing(&self) -> bool {
        self.learned_routing
    }
}
