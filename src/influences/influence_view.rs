use serde::Serialize;

use crate::influences::{influence::Influence, influence_evidence::InfluenceEvidence};

/// An influence with what is known of its effects, for inspection.
#[derive(Debug, Serialize)]
pub(crate) struct InfluenceView<'a> {
    influence: &'a Influence,
    attribution: &'static str,
    evidence: InfluenceEvidence,
}

impl<'a> InfluenceView<'a> {
    pub(crate) fn new(influence: &'a Influence, evidence: InfluenceEvidence) -> Self {
        Self {
            influence,
            attribution: evidence.attribution(),
            evidence,
        }
    }
}
