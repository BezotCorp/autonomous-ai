use serde::{Deserialize, Serialize};

/// Why the selector chose a module: the features describing the situation,
/// the expected success estimated from past experiences and its variance,
/// the exploratory
/// sample actually drawn and the competing candidates.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SelectionRecord {
    features: Vec<String>,
    expected_success: f64,
    success_variance: f64,
    sampled_success: f64,
    expected_cost_seconds: f64,
    utility: f64,
    best_expected_utility: f64,
    candidates: Vec<(String, f64)>,
}

impl SelectionRecord {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        features: Vec<String>,
        expected_success: f64,
        success_variance: f64,
        sampled_success: f64,
        expected_cost_seconds: f64,
        utility: f64,
        best_expected_utility: f64,
        candidates: Vec<(String, f64)>,
    ) -> Self {
        Self {
            features,
            expected_success,
            success_variance,
            sampled_success,
            expected_cost_seconds,
            utility,
            best_expected_utility,
            candidates,
        }
    }

    /// Highest expected utility (posterior mean minus expected cost) among
    /// the candidates, unaffected by the exploratory sampling.
    pub(crate) fn best_expected_utility(&self) -> f64 {
        self.best_expected_utility
    }

    pub(crate) fn features(&self) -> &[String] {
        &self.features
    }
}
