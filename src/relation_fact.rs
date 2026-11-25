use serde::Serialize;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct RelationFact {
    predicate: String,
    arguments: Vec<String>,
}

impl RelationFact {
    pub(crate) fn new(predicate: impl Into<String>, arguments: Vec<String>) -> Self {
        Self {
            predicate: predicate.into(),
            arguments,
        }
    }

    pub(crate) fn predicate(&self) -> &str {
        &self.predicate
    }

    pub(crate) fn arguments(&self) -> &[String] {
        &self.arguments
    }
}
