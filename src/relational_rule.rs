use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::relation_pattern::RelationPattern;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct RelationalRule {
    id: String,
    requires: Vec<RelationPattern>,
    concludes: RelationPattern,
    explanation: String,
}

impl RelationalRule {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn requires(&self) -> &[RelationPattern] {
        &self.requires
    }

    pub(crate) fn concludes(&self) -> &RelationPattern {
        &self.concludes
    }

    pub(crate) fn explanation(&self) -> &str {
        &self.explanation
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty()
            || self.explanation.trim().is_empty()
            || self.requires.is_empty()
            || !self.concludes.is_valid()
            || self.requires.iter().any(|pattern| !pattern.is_valid())
        {
            return Err(format!("Invalid relational rule: {}", self.id,));
        }

        let variables: BTreeSet<_> = self
            .requires
            .iter()
            .flat_map(|pattern| pattern.arguments())
            .filter(|argument| argument.starts_with('?'))
            .cloned()
            .collect();

        for argument in self.concludes.arguments() {
            if argument.starts_with('?') && !variables.contains(argument) {
                return Err(format!(
                    "Unbound variable {argument} in relational rule {}",
                    self.id,
                ));
            }
        }

        Ok(())
    }
}
