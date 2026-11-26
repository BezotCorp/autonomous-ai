use crate::{
    consumables::resource_meter::ResourceMeter,
    engine::{candidate::Candidate, learning_parameters::LearningParameters},
    mathematics::{beta_distribution::BetaDistribution, random_source::RandomSource},
    memory::{
        candidate_evaluation::CandidateEvaluation, routing_memory::RoutingMemory,
        selection_record::SelectionRecord,
    },
};

/// Contextual bandit choosing the next module by Thompson sampling.
///
/// For each candidate, the success probability is estimated separately for
/// every feature of the situation (plus the module-wide statistic), each
/// estimate being a Beta posterior. A feature never observed for a module
/// keeps the prior, so that a new kind of situation reopens exploration
/// instead of inheriting a judgement formed in other situations. One sample
/// is drawn per estimate and the samples are averaged, then weighted by the
/// stake of acting. The expected costs are subtracted: the recent duration
/// of the module and the scarcity of the resources it is expected to
/// consume, given what remains of them. Sampling instead of taking the
/// mean keeps exploring modules whose usefulness is still uncertain.
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
        meter: &ResourceMeter,
        random: &mut RandomSource,
    ) -> Option<(usize, SelectionRecord)> {
        let evaluations: Vec<CandidateEvaluation> = candidates
            .iter()
            .map(|candidate| self.evaluate(candidate, meter, random))
            .collect();

        let best_expected = evaluations
            .iter()
            .map(CandidateEvaluation::expected_utility)
            .fold(f64::NEG_INFINITY, f64::max);

        let (position, evaluation) = evaluations
            .iter()
            .enumerate()
            .max_by(|left, right| left.1.utility().total_cmp(&right.1.utility()))?;

        let competitors = candidates
            .iter()
            .zip(&evaluations)
            .map(|(candidate, evaluation)| (candidate.name().to_owned(), evaluation.utility()))
            .collect();

        let infeasible = evaluations
            .iter()
            .filter(|evaluation| !evaluation.is_feasible())
            .count();

        Some((
            position,
            SelectionRecord::new(
                candidates[position].features().to_vec(),
                evaluation.clone(),
                best_expected,
                competitors,
                infeasible,
            ),
        ))
    }

    fn evaluate(
        &self,
        candidate: &Candidate,
        meter: &ResourceMeter,
        random: &mut RandomSource,
    ) -> CandidateEvaluation {
        let posteriors: Vec<BetaDistribution> = candidate
            .features()
            .iter()
            .map(String::as_str)
            .chain([RoutingMemory::GLOBAL_FEATURE])
            .map(|feature| {
                let statistic = self.routing.statistic(candidate.name(), feature);

                BetaDistribution::posterior(
                    self.parameters.prior_successes(),
                    self.parameters.prior_failures(),
                    statistic.successes(),
                    statistic.failures(),
                )
            })
            .collect();

        let count = posteriors.len() as f64;
        let expected = posteriors.iter().map(BetaDistribution::mean).sum::<f64>() / count;
        let variance = posteriors
            .iter()
            .map(BetaDistribution::variance)
            .sum::<f64>()
            / count;
        let sampled = posteriors
            .iter()
            .map(|posterior| posterior.sample(random))
            .sum::<f64>()
            / count;

        let seconds = self
            .routing
            .cost(candidate.name())
            .expected_seconds()
            .unwrap_or(0.0);
        let consumption = self.routing.expected_consumption(candidate.name());
        let scarcity = meter.scarcity_cost(&consumption);
        let cost = self.parameters.cost_weight_per_second() * seconds
            + if scarcity.is_finite() {
                self.parameters.scarcity_weight() * scarcity
            } else {
                f64::INFINITY
            };

        CandidateEvaluation::new(
            candidate.stake(),
            expected,
            variance,
            sampled,
            seconds,
            consumption,
            scarcity,
            cost,
        )
    }
}
