use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    mathematics::beta_distribution::BetaDistribution,
    memory::{
        correction_journal::CorrectionJournal, cost_statistic::CostStatistic,
        dream_memory::DreamMemory, episode_record::EpisodeRecord,
        experience_memory::ExperienceMemory, invocation_outcome::InvocationOutcome,
        moving_average::MovingAverage, routing_statistic::RoutingStatistic,
    },
    planning::{plan::Plan, plan_status::PlanStatus, strategy_statistic::StrategyStatistic},
};

/// Procedural know-how: which modules turned out useful in which kinds of
/// situations, at which cost in time and resources, which information they
/// usually need first, and which strategies worked for which kinds of
/// goals. It is entirely derived from the experience register and can
/// therefore be recomputed at any time.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct RoutingMemory {
    statistics: BTreeMap<String, BTreeMap<String, RoutingStatistic>>,
    costs: BTreeMap<String, CostStatistic>,
    consumption: BTreeMap<String, BTreeMap<String, MovingAverage>>,
    prerequisites: BTreeMap<String, BTreeMap<String, u64>>,
    strategies: BTreeMap<String, BTreeMap<String, StrategyStatistic>>,
    simulated: BTreeMap<String, BTreeMap<String, RoutingStatistic>>,
    episodes_learned: usize,
    dreams_learned: usize,
}

impl RoutingMemory {
    /// Feature shared by every situation, giving a module-wide estimate.
    pub(crate) const GLOBAL_FEATURE: &'static str = "*";

    /// Relearns everything: real experiences, with the outcomes re-examined
    /// since, then dreams, kept apart as simulated experience.
    pub(crate) fn rebuild(
        experiences: &ExperienceMemory,
        dreams: &DreamMemory,
        corrections: &CorrectionJournal,
    ) -> Self {
        let mut memory = Self::default();

        for episode in experiences.episodes() {
            memory.learn_with(episode, corrections.reassessments());
        }

        for dream in dreams.dreams() {
            for episode in dream.scenes().iter().filter_map(|scene| scene.episode()) {
                memory.learn_simulated(episode);
            }
        }

        memory
    }

    pub(crate) fn learn(&mut self, episode: &EpisodeRecord) {
        self.learn_with(episode, &BTreeMap::new());
    }

    /// Learns from a dream: which modules proved useful in simulation and
    /// which strategies reached their goals. Kept apart from real outcomes.
    pub(crate) fn learn_simulated(&mut self, episode: &EpisodeRecord) {
        let questions_asked = !episode.questions().is_empty();

        for invocation in episode.invocations() {
            let success = invocation.outcome().is_success(questions_asked);
            let statistics = self
                .simulated
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
                .observe_simulated(record.achieved(), record.actual_cost());
        }

        self.dreams_learned += 1;
    }

    /// Posterior success probability of a module in situations sharing a
    /// feature, from real outcomes only. What dreams showed is no evidence
    /// about reality: it acts through the influences the unconscious draws
    /// from it, whose real consequences are learned in turn.
    pub(crate) fn estimate(
        &self,
        module: &str,
        feature: &str,
        prior: (f64, f64),
    ) -> BetaDistribution {
        let real = self.statistic(module, feature);

        BetaDistribution::posterior(prior.0, prior.1, real.successes(), real.failures())
    }

    fn learn_with(
        &mut self,
        episode: &EpisodeRecord,
        reassessments: &BTreeMap<(uuid::Uuid, usize), InvocationOutcome>,
    ) {
        let questions_asked = !episode.questions().is_empty();

        for invocation in episode.invocations() {
            let outcome = reassessments
                .get(&(episode.id(), invocation.step()))
                .copied()
                .unwrap_or(invocation.outcome());
            let success = outcome.is_success(questions_asked);
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
