use serde::Serialize;

use crate::mathematics::random_source::RandomSource;

/// Beta distribution describing the uncertain probability of a binary
/// outcome. It is the conjugate posterior of Bernoulli observations: the
/// prior parameters are explicit and the observed counts are added to them.
#[derive(Clone, Copy, Debug, Serialize)]
pub(crate) struct BetaDistribution {
    alpha: f64,
    beta: f64,
}

impl BetaDistribution {
    pub(crate) fn posterior(
        prior_alpha: f64,
        prior_beta: f64,
        successes: u64,
        failures: u64,
    ) -> Self {
        Self {
            alpha: prior_alpha + successes as f64,
            beta: prior_beta + failures as f64,
        }
    }

    /// The same distribution at a temperature: `Beta(α/T, β/T)` keeps the
    /// mean and widens (T > 1) or narrows (T < 1) the uncertainty.
    pub(crate) fn tempered(&self, temperature: f64) -> Self {
        if temperature == 1.0 || !(temperature.is_finite() && temperature > 0.0) {
            return *self;
        }

        Self {
            alpha: (self.alpha / temperature).max(1e-3),
            beta: (self.beta / temperature).max(1e-3),
        }
    }

    pub(crate) fn mean(&self) -> f64 {
        self.alpha / (self.alpha + self.beta)
    }

    pub(crate) fn variance(&self) -> f64 {
        let total = self.alpha + self.beta;

        self.alpha * self.beta / (total * total * (total + 1.0))
    }

    pub(crate) fn sample(&self, random: &mut RandomSource) -> f64 {
        let left = random.gamma(self.alpha);
        let right = random.gamma(self.beta);

        left / (left + right)
    }
}

#[cfg(test)]
mod tests {
    use super::BetaDistribution;
    use crate::mathematics::random_source::RandomSource;

    #[test]
    fn posterior_concentrates_with_evidence() {
        let weak = BetaDistribution::posterior(1.0, 1.0, 1, 0);
        let strong = BetaDistribution::posterior(1.0, 1.0, 90, 10);

        assert!(strong.variance() < weak.variance());
        assert!((strong.mean() - 91.0 / 102.0).abs() < 1e-12);

        let mut random = RandomSource::new(3);
        let sample = strong.sample(&mut random);

        assert!(sample > 0.0 && sample < 1.0);
    }
}
