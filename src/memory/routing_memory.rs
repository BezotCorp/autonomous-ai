use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    memory::{
        cost_statistic::CostStatistic, episode_record::EpisodeRecord,
        experience_memory::ExperienceMemory, moving_average::MovingAverage,
        routing_statistic::RoutingStatistic,
    },
    planning::{plan::Plan, plan_status::PlanStatus, strategy_statistic::StrategyStatistic},
};

/// Procedural know-how: which modules turned out useful in which kinds of
/// situations, at which cost in time and resources, which information they
/// usually need first, and which strategies worked for which kinds of
/// goals. It is entirely derived from the experience register and can
/// therefore be recomputed at any time.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct RoutingMemory {
    statistics: BTreeMap<String, BTreeMap<String, RoutingStatistic>>,
    costs: BTreeMap<String, CostStatistic>,
    consumption: BTreeMap<String, BTreeMap<String, MovingAverage>>,
    prerequisites: BTreeMap<String, BTreeMap<String, u64>>,
    strategies: BTreeMap<String, BTreeMap<String, StrategyStatistic>>,
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

            // Every resource the module ever consumed is updated, with zero
            // when it did not consume it this time.
            let consumption = self
                .consumption
                .entry(invocation.module().to_owned())
                .or_default();

            for resource in invocation.usage().consumption().keys() {
                consumption.entry(resource.clone()).or_default();
            }

            for (resource, average) in consumption.iter_mut() {
                average.observe(
                    invocation
                        .usage()
                        .consumption()
                        .get(resource)
                        .copied()
                        .unwrap_or(0.0),
                );
            }

            let prerequisites = self
                .prerequisites
                .entry(invocation.module().to_owned())
                .or_default();

            for feature in invocation.produced_features() {
                if let Some(signature) = feature.strip_prefix("need:") {
                    *prerequisites.entry(signature.to_owned()).or_default() += 1;
                }
            }
        }

        for record in episode.plans() {
            if record.status() == PlanStatus::Active {
                continue;
            }

            self.strategies
                .entry(Plan::goal_key(record.plan().goals()))
                .or_default()
                .entry(record.plan().strategy().to_owned())
                .or_default()
                .observe(
                    record.achieved(),
                    record.actual_cost(),
                    record.plan().estimate().cost(),
                );
        }

        self.episodes_learned += 1;
    }

    /// Units of each resource the module is expected to consume per invocation.
    pub(crate) fn expected_consumption(&self, module: &str) -> BTreeMap<String, f64> {
        self.consumption
            .get(module)
            .into_iter()
            .flatten()
            .filter_map(|(resource, average)| Some((resource.clone(), average.value()?)))
            .filter(|(_, units)| *units > 0.0)
            .collect()
    }

    /// Needs the module formulated in at least the given share of its
    /// invocations: information it usually requires before it can act.
    pub(crate) fn prerequisites(&self, module: &str, minimum_share: f64) -> Vec<String> {
        let invocations = self.cost(module).invocations();

        if invocations == 0 {
            return Vec::new();
        }

        self.prerequisites
            .get(module)
            .into_iter()
            .flatten()
            .filter(|(_, count)| **count as f64 / invocations as f64 >= minimum_share)
            .map(|(signature, _)| signature.clone())
            .collect()
    }

    pub(crate) fn strategy(&self, goal_key: &str, strategy: &str) -> Option<StrategyStatistic> {
        self.strategies.get(goal_key)?.get(strategy).copied()
    }

    pub(crate) fn strategies(&self) -> &BTreeMap<String, BTreeMap<String, StrategyStatistic>> {
        &self.strategies
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
