use serde::{Deserialize, Serialize};

use crate::{memory::impasse_kind::ImpasseKind, representation::statement::Statement};

/// Diagnosis of a need left unsatisfied, recorded so that blockages can be
/// recognised and compared across episodes.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ImpasseRecord {
    need: Statement,
    kind: ImpasseKind,
}

impl ImpasseRecord {
    pub(crate) fn new(need: Statement, kind: ImpasseKind) -> Self {
        Self { need, kind }
    }

    pub(crate) fn need(&self) -> &Statement {
        &self.need
    }

    pub(crate) fn kind(&self) -> &ImpasseKind {
        &self.kind
    }
}
