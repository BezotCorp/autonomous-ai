use serde::Serialize;

use crate::symbolic_rule::SymbolicRule;

#[derive(Debug, Serialize)]
pub(crate) struct SymbolicDeduction {
    rule: String,
    premises: Vec<String>,
    conclusion: String,
    explanation: String,
}

impl SymbolicDeduction {
    pub(crate) fn conclusion(&self) -> &str {
        &self.conclusion
    }

    pub(crate) fn new(rule: &SymbolicRule) -> Self {
        Self {
            rule: rule.id().to_owned(),
            premises: rule.requires().to_vec(),
            conclusion: rule.concludes().to_owned(),
            explanation: rule.explanation().to_owned(),
        }
    }
}
