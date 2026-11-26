use serde::{Deserialize, Serialize};

/// Conditions under which an episode ran, needed to compare experiments.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct EpisodeConfiguration {
    seed: u64,
    acquired_knowledge: bool,
    learned_routing: bool,
}

impl EpisodeConfiguration {
    pub(crate) fn new(seed: u64, acquired_knowledge: bool, learned_routing: bool) -> Self {
        Self {
            seed,
            acquired_knowledge,
            learned_routing,
        }
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
