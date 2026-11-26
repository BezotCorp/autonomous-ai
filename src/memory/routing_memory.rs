use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::memory::{
    cost_statistic::CostStatistic, episode_record::EpisodeRecord,
    experience_memory::ExperienceMemory, routing_statistic::RoutingStatistic,
};

/// Procedural know-how: which modules turned out useful in which kinds of
/// situations, and at which cost. It is entirely derived from the
/// experience register and can therefore be recomputed at any time.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct RoutingMemory {
    statistics: BTreeMap<String, BTreeMap<String, RoutingStatistic>>,
    costs: BTreeMap<String, CostStatistic>,
    episodes_learned: usize,
}

impl RoutingMemory {
    /// Feature shared by every situation, giving a module-wide estimate.
    pub(crate) const GLOBAL_FEATURE: &'static str = "*";

    pub(crate) fn rebuild(experiences: &ExperienceMemory) -> Self {
        let mut memory = Self::default();

        for episode in experiences.episodes() {
            memory.learn(episode);
        }

        memory
    }

    pub(crate) fn learn(&mut self, episode: &EpisodeRecord) {
        let questions_asked = !episode.questions().is_empty();

        for invocation in episode.invocations() {
            let success = invocation.outcome().is_success(questions_asked);
            let statistics = self
                .statistics
                .entry(invocation.module().to_owned())
                .or_default();

            for feature in invocation
                .selection()
                .features()
                .iter()
                .map(String::as_str)
                .chain([Self::GLOBAL_FEATURE])
            {
                statistics
                    .entry(feature.to_owned())
                    .or_default()
                    .observe(success);
            }

            self.costs
                .entry(invocation.module().to_owned())
                .or_default()
                .observe(invocation.duration_micros());
        }

        self.episodes_learned += 1;
    }

    pub(crate) fn statistic(&self, module: &str, feature: &str) -> RoutingStatistic {
        self.statistics
            .get(module)
            .and_then(|statistics| statistics.get(feature))
            .copied()
            .unwrap_or_default()
    }

    pub(crate) fn cost(&self, module: &str) -> CostStatistic {
        self.costs.get(module).copied().unwrap_or_default()
    }

    pub(crate) fn episodes_learned(&self) -> usize {
        self.episodes_learned
    }
}
