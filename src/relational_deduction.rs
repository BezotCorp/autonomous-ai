use serde::Serialize;

use crate::relation_fact::RelationFact;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct RelationalDeduction {
    rule: String,
    premises: Vec<RelationFact>,
    conclusion: RelationFact,
    explanation: String,
}

impl RelationalDeduction {
    pub(crate) fn new(
        rule: String,
        premises: Vec<RelationFact>,
        conclusion: RelationFact,
        explanation: String,
    ) -> Self {
        Self {
            rule,
            premises,
            conclusion,
            explanation,
        }
    }
}
