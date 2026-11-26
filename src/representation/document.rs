use serde::{Deserialize, Serialize};

/// Raw textual material, kept distinct from the knowledge interpreted from it.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct Document {
    identifier: String,
    language: Option<String>,
    text: String,
}

impl Document {
    pub(crate) fn new(
        identifier: impl Into<String>,
        language: Option<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            identifier: identifier.into(),
            language,
            text: text.into(),
        }
    }

    pub(crate) fn identifier(&self) -> &str {
        &self.identifier
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }
}
