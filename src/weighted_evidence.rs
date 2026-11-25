use serde::Serialize;

use crate::relation_fact::RelationFact;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct WeightedEvidence {
    subject: Vec<String>,
    score: f64,
    observations: Vec<RelationFact>,
}

impl WeightedEvidence {
    pub(crate) fn new(subject: Vec<String>, score: f64, observations: Vec<RelationFact>) -> Self {
        Self {
            subject,
            score,
            observations,
        }
    }
}
