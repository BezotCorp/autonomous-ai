use serde::Serialize;

use crate::relation_fact::RelationFact;
use crate::relational_deduction::RelationalDeduction;
use crate::weighted_evidence::WeightedEvidence;

#[derive(Debug, Serialize)]
pub(crate) struct CognitionResult {
    relations: Vec<RelationFact>,
    deductions: Vec<RelationalDeduction>,
    evidence_scores: Vec<WeightedEvidence>,
}

impl CognitionResult {
    pub(crate) fn new(
        relations: Vec<RelationFact>,
        deductions: Vec<RelationalDeduction>,
        evidence_scores: Vec<WeightedEvidence>,
    ) -> Self {
        Self {
            relations,
            deductions,
            evidence_scores,
        }
    }
}
