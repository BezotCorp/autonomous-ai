use crate::{
    engine::{candidate::Candidate, learning_parameters::LearningParameters},
    mathematics::{beta_distribution::BetaDistribution, random_source::RandomSource},
    memory::{routing_memory::RoutingMemory, selection_record::SelectionRecord},
};

/// Contextual bandit choosing the next module by Thompson sampling.
///
/// For each candidate, the success probability is estimated separately for
/// every feature of the situation (plus the module-wide statistic), each
/// estimate being a Beta posterior. A feature never observed for a module
/// keeps the prior, so that a new kind of situation reopens exploration
/// instead of inheriting a judgement formed in other situations. One sample
/// is drawn per estimate and the samples are averaged; the expected cost is
/// then subtracted. Sampling instead of taking the mean keeps exploring
/// modules whose usefulness is still uncertain.
pub(crate) struct ModuleSelector<'a> {
    routing: &'a RoutingMemory,
    parameters: &'a LearningParameters,
}

impl<'a> ModuleSelector<'a> {
    pub(crate) fn new(routing: &'a RoutingMemory, parameters: &'a LearningParameters) -> Self {
        Self {
            routing,
            parameters,
        }
    }

    /// Returns the position of the chosen candidate and the trace of the choice.
    pub(crate) fn select(
        &self,
        candidates: &[Candidate],
        random: &mut RandomSource,
    ) -> Option<(usize, SelectionRecord)> {
        let mut evaluations = Vec::with_capacity(candidates.len());
        let mut best_expected = f64::NEG_INFINITY;

        for candidate in candidates {
            let mut posteriors = Vec::new();

            for feature in candidate
                .features()
                .iter()
                .map(String::as_str)
                .chain([RoutingMemory::GLOBAL_FEATURE])
            {
                let statistic = self.routing.statistic(candidate.name(), feature);

                posteriors.push(self.posterior(statistic.successes(), statistic.failures()));
            }

            let count = posteriors.len() as f64;
            let expected = posteriors.iter().map(BetaDistribution::mean).sum::<f64>() / count;
            let uncertainty = posteriors
                .iter()
                .map(BetaDistribution::variance)
                .sum::<f64>()
                / count;
            let sampled = posteriors
                .iter()
                .map(|posterior| posterior.sample(random))
                .sum::<f64>()
                / count;

            let cost = self
                .routing
                .cost(candidate.name())
                .mean_seconds()
                .unwrap_or(0.0);

            let utility = sampled - self.parameters.cost_weight_per_second() * cost;

            evaluations.push((expected, uncertainty, sampled, cost, utility));
            best_expected =
                best_expected.max(expected - self.parameters.cost_weight_per_second() * cost);
        }

        let (position, (expected, uncertainty, sampled, cost, utility)) = evaluations
            .iter()
            .copied()
            .enumerate()
            .max_by(|left, right| left.1.4.total_cmp(&right.1.4))?;

        let competitors = candidates
            .iter()
            .zip(&evaluations)
            .map(|(candidate, evaluation)| (candidate.name().to_owned(), evaluation.4))
            .collect();

        Some((
            position,
            SelectionRecord::new(
                candidates[position].features().to_vec(),
                expected,
                uncertainty,
                sampled,
                cost,
                utility,
                best_expected,
                competitors,
            ),
        ))
    }

    fn posterior(&self, successes: u64, failures: u64) -> BetaDistribution {
        BetaDistribution::posterior(
            self.parameters.prior_successes(),
            self.parameters.prior_failures(),
            successes,
            failures,
        )
    }
}
