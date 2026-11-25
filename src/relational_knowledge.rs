use serde::Deserialize;

use crate::{
    evidence_weight::EvidenceWeight, lexical_rule::LexicalRule, relational_rule::RelationalRule,
};

#[derive(Debug, Deserialize)]
pub(crate) struct RelationalKnowledge {
    relational_rules: Vec<RelationalRule>,
    lexical_rules: Vec<LexicalRule>,
    evidence_weights: Vec<EvidenceWeight>,
}

impl RelationalKnowledge {
    pub(crate) fn into_parts(self) -> (Vec<RelationalRule>, Vec<LexicalRule>, Vec<EvidenceWeight>) {
        (
            self.relational_rules,
            self.lexical_rules,
            self.evidence_weights,
        )
    }
}
