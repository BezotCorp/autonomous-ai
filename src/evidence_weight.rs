use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct EvidenceWeight {
    predicate: String,
    subject_indices: Vec<usize>,
    weight: f64,
}

impl EvidenceWeight {
    pub(crate) fn predicate(&self) -> &str {
        &self.predicate
    }

    pub(crate) fn subject_indices(&self) -> &[usize] {
        &self.subject_indices
    }

    pub(crate) fn weight(&self) -> f64 {
        self.weight
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.predicate.trim().is_empty()
            || self.subject_indices.is_empty()
            || !self.weight.is_finite()
        {
            return Err(format!(
                "Invalid evidence weight for predicate: {}",
                self.predicate,
            ));
        }

        Ok(())
    }
}
