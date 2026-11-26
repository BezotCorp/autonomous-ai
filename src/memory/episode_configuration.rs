use serde::{Deserialize, Serialize};

use crate::engine::learning_parameters::LearningParameters;

/// Conditions under which an episode ran, needed to compare experiments.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct EpisodeConfiguration {
    seed: u64,
    acquired_knowledge: bool,
    learned_routing: bool,
    cost_weight_per_second: f64,
    minimum_utility: f64,
}

impl EpisodeConfiguration {
    pub(crate) fn new(seed: u64, acquired_knowledge: bool, learned_routing: bool) -> Self {
        Self {
            seed,
            acquired_knowledge,
            learned_routing,
            cost_weight_per_second: 0.0,
            minimum_utility: 0.0,
        }
    }

    /// Records the decision parameters the episode ran with.
    pub(crate) fn with_parameters(mut self, parameters: &LearningParameters) -> Self {
        self.cost_weight_per_second = parameters.cost_weight_per_second();
        self.minimum_utility = parameters.minimum_utility();
        self
    }

    /// Label grouping comparable episodes.
    pub(crate) fn label(&self) -> String {
        format!(
            "knowledge={} routing={} cost_weight={} minimum_utility={}",
            self.acquired_knowledge,
            self.learned_routing,
            self.cost_weight_per_second,
            self.minimum_utility
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
