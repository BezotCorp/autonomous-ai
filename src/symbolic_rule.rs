use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SymbolicRule {
    id: String,
    requires: Vec<String>,
    concludes: String,
    explanation: String,
}

impl SymbolicRule {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn requires(&self) -> &[String] {
        &self.requires
    }

    pub(crate) fn concludes(&self) -> &str {
        &self.concludes
    }

    pub(crate) fn explanation(&self) -> &str {
        &self.explanation
    }
}
