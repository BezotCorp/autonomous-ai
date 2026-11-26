use std::collections::BTreeMap;

use uuid::Uuid;

use crate::{
    consumables::resource_meter::ResourceMeter,
    engine::{
        candidate::Candidate, learning_parameters::LearningParameters, modulation::Modulation,
    },
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
/// feature is drawn and its posterior sampled (a mixture: the uncertainty of
/// each view is preserved), then weighted by the stake of acting. The expected costs are subtracted: the recent duration
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
    ///
    /// `preferences` are the influences of the unconscious bearing on some
    /// modules: each brings its own estimate, learned from the real
    /// consequences of the decisions it changed (uniform while untested).
    /// One that favours a module may raise its sampled utility, one that
    /// questions it may lower it: an alternative thus gets a real chance,
    /// without ever being imposed. The choice that would have been made
    /// without them is computed as well, to know which ones were decisive.
    ///
    /// The modulation of the moment sets the temperature of the sampled
    /// posteriors, the weight of costs and the chance that each influence
    /// is admitted at all.
    pub(crate) fn select(
        &self,
        candidates: &[Candidate],
        meter: &ResourceMeter,
        random: &mut RandomSource,
        preferences: &BTreeMap<String, Vec<(Uuid, bool, BetaDistribution)>>,
        modulation: &Modulation,
    ) -> Option<(usize, SelectionRecord)> {
        let evaluations: Vec<CandidateEvaluation> = candidates
            .iter()
            .map(|candidate| self.evaluate(candidate, meter, random, modulation))
            .collect();

        let mut influenced = Vec::with_capacity(candidates.len());

        for (candidate, evaluation) in candidates.iter().zip(&evaluations) {
            let mut utility = evaluation.utility();
            let mut applied = Vec::new();

            for (influence, favour, posterior) in
                preferences.get(candidate.name()).into_iter().flatten()
            {
                if modulation.admission() < 1.0 && random.uniform() >= modulation.admission() {
                    continue;
                }

                let suggested = candidate.stake() * posterior.sample(random) - evaluation.cost();

                if (*favour && suggested > utility) || (!*favour && suggested < utility) {
                    utility = suggested;
                    applied.push(*influence);
                }
            }

            influenced.push((utility, applied));
        }

        let best_expected = evaluations
            .iter()
            .map(CandidateEvaluation::expected_utility)
            .fold(f64::NEG_INFINITY, f64::max);

        let position = influenced
            .iter()
            .enumerate()
            .max_by(|left, right| left.1.0.total_cmp(&right.1.0))?
            .0;
        let habitual = evaluations
            .iter()
            .enumerate()
            .max_by(|left, right| left.1.utility().total_cmp(&right.1.utility()))?
            .0;

        let decisive = if position == habitual {
            Vec::new()
        } else {
            influenced[position]
                .1
                .iter()
                .chain(&influenced[habitual].1)
                .copied()
                .collect()
        };

        let competitors = candidates
            .iter()
            .zip(&influenced)
            .map(|(candidate, (utility, _))| (candidate.name().to_owned(), *utility))
            .collect();

        let infeasible = evaluations
            .iter()
            .filter(|evaluation| !evaluation.is_feasible())
            .count();

        Some((
            position,
            SelectionRecord::new(
                candidates[position].features().to_vec(),
                evaluations[position].clone(),
                best_expected,
                competitors,
                infeasible,
            )
            .with_influences(decisive),
        ))
    }

    fn evaluate(
        &self,
        candidate: &Candidate,
        meter: &ResourceMeter,
        random: &mut RandomSource,
        modulation: &Modulation,
    ) -> CandidateEvaluation {
        let posteriors: Vec<BetaDistribution> = candidate
            .features()
            .iter()
            .map(String::as_str)
            .chain([RoutingMemory::GLOBAL_FEATURE])
            .map(|feature| {
                self.routing.estimate(
                    candidate.name(),
                    feature,
                    (
                        self.parameters.prior_successes(),
                        self.parameters.prior_failures(),
                    ),
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
        // The features are a mixture of views on the situation: one of them
        // is drawn, then its posterior sampled. Averaging samples instead
        // would divide the uncertainty by the number of features and starve
        // never-tried alternatives of any chance.
        let drawn = ((random.uniform() * count) as usize).min(posteriors.len() - 1);
        let sampled = posteriors[drawn]
            .tempered(modulation.temperature())
            .sample(random);

        let seconds = self
            .routing
            .cost(candidate.name())
            .expected_seconds()
            .unwrap_or(0.0);
        let consumption = self.routing.expected_consumption(candidate.name());
        let scarcity = meter.scarcity_cost(&consumption);
        let cost = modulation.cost_factor()
            * (self.parameters.cost_weight_per_second() * seconds
                + if scarcity.is_finite() {
                    self.parameters.scarcity_weight() * scarcity
                } else {
                    f64::INFINITY
                });

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
