use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::individuality::birth_parameters::BirthParameters;

/// Who an individual is: its identifier, its name, when and from what it
/// was born, and from which individuals it was duplicated, if any.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Identity {
    id: Uuid,
    name: String,
    born_at: DateTime<Utc>,
    birth: BirthParameters,
    lineage: Vec<Uuid>,
}

impl Identity {
    pub(crate) fn new(name: String, birth: BirthParameters) -> Self {
        Self {
            id: Uuid::now_v7(),
            name,
            born_at: Utc::now(),
            birth,
            lineage: Vec::new(),
        }
    }

    /// Identity of a duplicate: same birth, new identifier, and the
    /// original recorded in the lineage.
    pub(crate) fn duplicate(&self, name: String) -> Self {
        let mut lineage = self.lineage.clone();

        lineage.push(self.id);

        Self {
            id: Uuid::now_v7(),
            name,
            born_at: self.born_at,
            birth: self.birth.clone(),
            lineage,
        }
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn birth(&self) -> &BirthParameters {
        &self.birth
    }
}
