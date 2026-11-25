use std::collections::BTreeMap;

use crate::relation_fact::RelationFact;

pub(crate) struct RuleMatch {
    bindings: BTreeMap<String, String>,
    premises: Vec<RelationFact>,
}

impl RuleMatch {
    pub(crate) fn new(bindings: BTreeMap<String, String>, premises: Vec<RelationFact>) -> Self {
        Self { bindings, premises }
    }

    pub(crate) fn bindings(&self) -> &BTreeMap<String, String> {
        &self.bindings
    }

    pub(crate) fn into_premises(self) -> Vec<RelationFact> {
        self.premises
    }
}
