use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct LexicalRule {
    id: String,
    phrases: Vec<String>,
    capability: String,
}

impl LexicalRule {
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn phrases(&self) -> &[String] {
        &self.phrases
    }

    pub(crate) fn capability(&self) -> &str {
        &self.capability
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty()
            || self.capability.trim().is_empty()
            || self.phrases.is_empty()
            || self.phrases.iter().any(|phrase| phrase.trim().is_empty())
        {
            return Err(format!("Invalid lexical rule: {}", self.id));
        }

        Ok(())
    }
}
