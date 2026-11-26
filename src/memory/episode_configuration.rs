use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    acquisition::access_mode::AccessMode, engine::learning_parameters::LearningParameters,
    memory::activity::Activity,
};

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
    activity: Activity,
    access: AccessMode,
    excluded_modules: BTreeSet<String>,
    acquisition_cache: bool,
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
            activity: Activity::Action,
            access: AccessMode::Live,
            excluded_modules: BTreeSet::new(),
            acquisition_cache: true,
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

    pub(crate) fn with_activity(mut self, activity: Activity) -> Self {
        self.activity = activity;
        self
    }

    /// Replays recorded data instead of reaching the environment.
    pub(crate) fn with_access(mut self, access: AccessMode) -> Self {
        self.access = access;
        self
    }

    /// Capabilities the episode must do without.
    pub(crate) fn without_modules(mut self, modules: BTreeSet<String>) -> Self {
        self.excluded_modules = modules;
        self
    }

    /// Disables the reuse of acquired data, to tell technical gains apart
    /// from cognitive ones.
    pub(crate) fn without_acquisition_cache(mut self) -> Self {
        self.acquisition_cache = false;
        self
    }

    pub(crate) fn activity(&self) -> Activity {
        self.activity
    }

    pub(crate) fn access(&self) -> AccessMode {
        self.access
    }

    pub(crate) fn excludes(&self, module: &str) -> bool {
        self.excluded_modules.contains(module)
    }

    pub(crate) fn acquisition_cache(&self) -> bool {
        self.acquisition_cache
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
            "activity={} knowledge={} routing={} cache={} cost_weight={} scarcity_weight={} minimum_utility={}{}",
            self.activity.label(),
            self.acquired_knowledge,
            self.learned_routing,
            self.acquisition_cache,
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
