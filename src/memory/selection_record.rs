use serde::{Deserialize, Serialize};

use crate::memory::candidate_evaluation::CandidateEvaluation;

/// Why the selector chose a module: the features describing the situation,
/// how the chosen module was valued, the best expected utility among all
/// candidates (which decides whether acting is worth it at all), the
/// utility of each competitor and how many required an exhausted resource.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SelectionRecord {
    features: Vec<String>,
    evaluation: CandidateEvaluation,
    best_expected_utility: f64,
    candidates: Vec<(String, f64)>,
    infeasible: usize,
}

impl SelectionRecord {
    pub(crate) fn new(
        features: Vec<String>,
        evaluation: CandidateEvaluation,
        best_expected_utility: f64,
        candidates: Vec<(String, f64)>,
        infeasible: usize,
    ) -> Self {
        Self {
            features,
            evaluation,
            best_expected_utility,
            candidates,
            infeasible,
        }
    }

    /// Highest expected utility (posterior mean minus expected costs) among
    /// the candidates, unaffected by the exploratory sampling.
    pub(crate) fn best_expected_utility(&self) -> f64 {
        self.best_expected_utility
    }

    pub(crate) fn features(&self) -> &[String] {
        &self.features
    }

    pub(crate) fn evaluation(&self) -> &CandidateEvaluation {
        &self.evaluation
    }

    pub(crate) fn infeasible(&self) -> usize {
        self.infeasible
    }
}
