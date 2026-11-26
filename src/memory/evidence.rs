use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::representation::{provenance::Provenance, statement::Statement};

/// One justification of a persisted statement.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Evidence {
    episode: Uuid,
    provenance: Option<Provenance>,
    premises: Vec<Statement>,
}

impl Evidence {
    pub(crate) fn new(
        episode: Uuid,
        provenance: Option<Provenance>,
        premises: Vec<Statement>,
    ) -> Self {
        Self {
            episode,
            provenance,
            premises,
        }
    }

    pub(crate) fn provenance(&self) -> Option<&Provenance> {
        self.provenance.as_ref()
    }

    pub(crate) fn premises(&self) -> &[Statement] {
        &self.premises
    }

    /// Two evidences are redundant when they come from the same source
    /// revision or from the same premises.
    pub(crate) fn is_redundant_with(&self, other: &Self) -> bool {
        match (&self.provenance, &other.provenance) {
            (Some(left), Some(right)) => {
                left.source() == right.source()
                    && left.scope() == right.scope()
                    && left.revision() == right.revision()
            }
            (None, None) => self.premises == other.premises,
            _ => false,
        }
    }
}
