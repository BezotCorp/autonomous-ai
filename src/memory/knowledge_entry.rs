use serde::{Deserialize, Serialize};

use crate::{
    memory::{evidence::Evidence, standing::Standing},
    representation::statement::Statement,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct KnowledgeEntry {
    statement: Statement,
    standing: Standing,
    evidence: Vec<Evidence>,
}

impl KnowledgeEntry {
    pub(crate) fn new(statement: Statement, standing: Standing) -> Self {
        Self {
            statement,
            standing,
            evidence: Vec::new(),
        }
    }

    pub(crate) fn statement(&self) -> &Statement {
        &self.statement
    }

    pub(crate) fn standing(&self) -> &Standing {
        &self.standing
    }

    pub(crate) fn evidence(&self) -> &[Evidence] {
        &self.evidence
    }

    /// The entry as it stood at an earlier instant, if it existed then.
    pub(crate) fn as_of(&self, instant: chrono::DateTime<chrono::Utc>) -> Option<Self> {
        let evidence: Vec<Evidence> = self
            .evidence
            .iter()
            .filter(|evidence| evidence.recorded_at() <= instant)
            .cloned()
            .collect();

        if evidence.is_empty() {
            return None;
        }

        let standing = match &self.standing {
            Standing::Obsolete { since, .. } if *since <= instant => self.standing.clone(),
            _ if evidence
                .iter()
                .any(|evidence| evidence.provenance().is_some()) =>
            {
                Standing::Observed
            }
            _ => Standing::Derived,
        };

        Some(Self {
            statement: self.statement.clone(),
            standing,
            evidence,
        })
    }

    pub(crate) fn set_standing(&mut self, standing: Standing) {
        self.standing = standing;
    }

    /// Adds a justification unless an equivalent one is already known.
    /// Returns whether the evidence was new.
    pub(crate) fn support(&mut self, evidence: Evidence) -> bool {
        if self
            .evidence
            .iter()
            .any(|known| known.is_redundant_with(&evidence))
        {
            return false;
        }

        self.evidence.push(evidence);
        true
    }
}
