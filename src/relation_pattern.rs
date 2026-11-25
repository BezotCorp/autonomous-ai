use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct RelationPattern {
    predicate: String,
    arguments: Vec<String>,
}

impl RelationPattern {
    pub(crate) fn predicate(&self) -> &str {
        &self.predicate
    }

    pub(crate) fn arguments(&self) -> &[String] {
        &self.arguments
    }

    pub(crate) fn is_valid(&self) -> bool {
        !self.predicate.trim().is_empty()
            && self.arguments.iter().all(|argument| {
                !argument.trim().is_empty() && (!argument.starts_with('?') || argument.len() > 1)
            })
    }
}
