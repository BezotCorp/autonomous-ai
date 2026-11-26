use std::collections::BTreeMap;

use crate::{
    cognition::module_descriptor::ModuleDescriptor,
    engine::learning_parameters::LearningParameters,
    mathematics::beta_distribution::BetaDistribution, memory::routing_memory::RoutingMemory,
    planning::strategy_statistic::StrategyStatistic,
};

/// What the intelligence knows about its own capabilities, for anticipating
/// before acting: what each module declares it can supply, and what
/// experience taught about its success, duration, consumption of resources
/// and the information it usually needs first; which kinds of information
/// the persistent memory can provide and how outdated they may be; which
/// strategies worked for which kinds of goals.
pub(crate) struct CapabilityModel<'a> {
    routing: &'a RoutingMemory,
    parameters: &'a LearningParameters,
    descriptors: Vec<ModuleDescriptor>,
    remembered: BTreeMap<String, f64>,
}

impl<'a> CapabilityModel<'a> {
    /// Share of its invocations in which a module must have formulated a
    /// need for that need to be anticipated as one of its prerequisites.
    const PREREQUISITE_SHARE: f64 = 0.25;

    pub(crate) fn new(
        routing: &'a RoutingMemory,
        parameters: &'a LearningParameters,
        descriptors: Vec<ModuleDescriptor>,
        remembered: BTreeMap<String, f64>,
    ) -> Self {
        Self {
            routing,
            parameters,
            descriptors,
            remembered,
        }
    }

    pub(crate) fn descriptors(&self) -> &[ModuleDescriptor] {
        &self.descriptors
    }

    /// Expected probability of success of the module in a situation
    /// described by these features (posterior means, the module-wide
    /// estimate included).
    pub(crate) fn success(&self, module: &str, features: &[String]) -> f64 {
        let estimates: Vec<f64> = features
            .iter()
            .map(String::as_str)
            .chain([RoutingMemory::GLOBAL_FEATURE])
            .map(|feature| {
                let statistic = self.routing.statistic(module, feature);

                BetaDistribution::posterior(
                    self.parameters.prior_successes(),
                    self.parameters.prior_failures(),
                    statistic.successes(),
                    statistic.failures(),
                )
                .mean()
            })
            .collect();

        estimates.iter().sum::<f64>() / estimates.len() as f64
    }

    pub(crate) fn seconds(&self, module: &str) -> f64 {
        self.routing.cost(module).expected_seconds().unwrap_or(0.0)
    }

    pub(crate) fn consumption(&self, module: &str) -> BTreeMap<String, f64> {
        self.routing.expected_consumption(module)
    }

    pub(crate) fn prerequisites(&self, module: &str) -> Vec<String> {
        self.routing.prerequisites(module, Self::PREREQUISITE_SHARE)
    }

    /// Staleness of the knowledge the memory holds for this signature, or
    /// `None` when it holds none.
    pub(crate) fn remembered(&self, signature: &str) -> Option<f64> {
        self.remembered.get(signature).copied()
    }

    pub(crate) fn strategy(&self, goal_key: &str, strategy: &str) -> Option<StrategyStatistic> {
        self.routing.strategy(goal_key, strategy)
    }
}
