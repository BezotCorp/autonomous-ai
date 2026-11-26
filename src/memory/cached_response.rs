use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Data received from an external source, kept to avoid acquiring it again:
/// either content addressed by an immutable identifier, or content of a
/// mutable resource with the validator allowing to ask whether it changed.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct CachedResponse {
    body: String,
    validator: Option<String>,
    stored_at: DateTime<Utc>,
    last_used_at: DateTime<Utc>,
}

impl CachedResponse {
    pub(crate) fn new(body: String, validator: Option<String>) -> Self {
        let now = Utc::now();

        Self {
            body,
            validator,
            stored_at: now,
            last_used_at: now,
        }
    }

    pub(crate) fn body(&self) -> &str {
        &self.body
    }

    pub(crate) fn validator(&self) -> Option<&str> {
        self.validator.as_deref()
    }

    pub(crate) fn last_used_at(&self) -> DateTime<Utc> {
        self.last_used_at
    }

    pub(crate) fn mark_used(&mut self, at: DateTime<Utc>) {
        self.last_used_at = at;
    }

    pub(crate) fn size(&self) -> usize {
        self.body.len() + self.validator.as_ref().map_or(0, String::len)
    }
}
