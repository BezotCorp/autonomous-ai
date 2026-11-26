use std::time::Duration;

/// Explicit parameters of the learning and decision algorithms. They are
/// configuration, not knowledge: nothing here is learned, and every learned
/// statistic lives in the routing memory instead.
#[derive(Clone, Debug)]
pub(crate) struct LearningParameters {
    /// Beta prior of the success probability of a module never observed in
    /// a situation. (1, 1) is the uniform prior: no preference at all.
    prior_successes: f64,
    prior_failures: f64,
    /// Penalty subtracted from the sampled success probability per second
    /// of expected execution time. 0 ignores costs entirely.
    cost_weight_per_second: f64,
    /// Expected utility below which no candidate is considered worth trying
    /// and the episode stops. The default never stops on this ground.
    minimum_utility: f64,
    /// Maximum number of module invocations in one episode.
    maximum_invocations: usize,
    /// Maximum wall-clock duration of one episode.
    maximum_duration: Duration,
}

impl Default for LearningParameters {
    fn default() -> Self {
        Self {
            prior_successes: 1.0,
            prior_failures: 1.0,
            cost_weight_per_second: 0.01,
            minimum_utility: 0.0,
            maximum_invocations: 64,
            maximum_duration: Duration::from_secs(600),
        }
    }
}

impl LearningParameters {
    pub(crate) fn with_maximum_invocations(mut self, maximum: usize) -> Self {
        self.maximum_invocations = maximum;
        self
    }

    pub(crate) fn with_cost_weight_per_second(mut self, weight: f64) -> Self {
        self.cost_weight_per_second = weight;
        self
    }

    pub(crate) fn with_minimum_utility(mut self, minimum: f64) -> Self {
        self.minimum_utility = minimum;
        self
    }

    pub(crate) fn prior_successes(&self) -> f64 {
        self.prior_successes
    }

    pub(crate) fn prior_failures(&self) -> f64 {
        self.prior_failures
    }

    pub(crate) fn cost_weight_per_second(&self) -> f64 {
        self.cost_weight_per_second
    }

    pub(crate) fn minimum_utility(&self) -> f64 {
        self.minimum_utility
    }

    pub(crate) fn maximum_invocations(&self) -> usize {
        self.maximum_invocations
    }

    pub(crate) fn maximum_duration(&self) -> Duration {
        self.maximum_duration
    }
}
