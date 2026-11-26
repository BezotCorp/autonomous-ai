use serde::{Deserialize, Serialize};

use crate::representation::{rule::Rule, statement::Statement};

/// Knowledge supplied before any experience: general statements and
/// implications written in the textual notation.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct InitialKnowledge {
    #[serde(default)]
    statements: Vec<Statement>,
    #[serde(default)]
    rules: Vec<Rule>,
}

impl InitialKnowledge {
    pub(crate) fn statements(&self) -> &[Statement] {
        &self.statements
    }

    pub(crate) fn rules(&self) -> &[Rule] {
        &self.rules
    }
}
