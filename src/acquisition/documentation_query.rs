use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct DocumentationQuery {
    name: String,
    expression: String,
}

impl DocumentationQuery {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn expression(&self) -> &str {
        &self.expression
    }
}
