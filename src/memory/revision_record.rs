use serde::{Deserialize, Serialize};

/// Trace of a contradiction and of the belief revision it triggered.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct RevisionRecord {
    step: usize,
    positive: String,
    negative: String,
    examined_causes: Vec<String>,
    withdrawn: Vec<String>,
    invalidated_results: usize,
    resolved: bool,
}

impl RevisionRecord {
    pub(crate) fn new(
        step: usize,
        positive: String,
        negative: String,
        examined_causes: Vec<String>,
        withdrawn: Vec<String>,
        invalidated_results: usize,
        resolved: bool,
    ) -> Self {
        Self {
            step,
            positive,
            negative,
            examined_causes,
            withdrawn,
            invalidated_results,
            resolved,
        }
    }
}
