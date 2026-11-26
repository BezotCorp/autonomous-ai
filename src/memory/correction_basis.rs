use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::representation::statement::Statement;

/// What a correction rests on. Corrections can themselves be questioned:
/// when their basis disappears, they can be reverted.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CorrectionBasis {
    /// Known facts that logically entail the correction.
    Proof { facts: Vec<Statement> },
    /// An internal experiment (dream) whose outcome supports it.
    Experiment { dream: Uuid },
    /// Real outcomes observed when it was at work.
    Outcomes {
        successes: u64,
        failures: u64,
        baseline: f64,
    },
    /// A contradiction or inconsistency within the memory itself.
    Inconsistency { detail: String },
}
