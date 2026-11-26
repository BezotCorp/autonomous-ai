use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Data received from an external source, kept to avoid acquiring it again:
/// either content addressed by an immutable identifier, or content of a
/// mutable resource with the validator allowing to ask whether it changed.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct CachedResponse {
    body: String,
    validator: Option<String>,
    immutable: bool,
    next: Option<String>,
    stored_at: DateTime<Utc>,
    last_used_at: DateTime<Utc>,
}

impl CachedResponse {
    /// Content addressed by an immutable identifier: valid at any time.
    pub(crate) fn immutable(body: String) -> Self {
        Self::new(body, None, true)
    }

    /// Content of a mutable resource, as acquired now.
    pub(crate) fn mutable(body: String, validator: Option<String>) -> Self {
        Self::new(body, validator, false)
    }

    fn new(body: String, validator: Option<String>, immutable: bool) -> Self {
        let now = Utc::now();

        Self {
            body,
            validator,
            immutable,
            next: None,
            stored_at: now,
            last_used_at: now,
        }
    }

    /// Location of the next page of a paginated answer.
    pub(crate) fn with_next(mut self, next: Option<String>) -> Self {
        self.next = next;
        self
    }

    pub(crate) fn next(&self) -> Option<&str> {
        self.next.as_deref()
    }

    pub(crate) fn is_immutable(&self) -> bool {
        self.immutable
    }

    pub(crate) fn stored_at(&self) -> DateTime<Utc> {
        self.stored_at
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

    pub(crate) fn revalidate(&mut self, validator: Option<String>) {
        if validator.is_some() {
            self.validator = validator;
        }
    }

    pub(crate) fn mark_used(&mut self, at: DateTime<Utc>) {
        self.last_used_at = at;
    }

    pub(crate) fn size(&self) -> usize {
        self.body.len() + self.validator.as_ref().map_or(0, String::len)
    }
}
