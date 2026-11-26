use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Origin of an acquired piece of information: which source produced it,
/// which scope was observed and which exact revision of that scope.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct Provenance {
    source: String,
    scope: Option<String>,
    revision: Option<String>,
    acquired_at: DateTime<Utc>,
}

impl Provenance {
    pub(crate) fn new(
        source: impl Into<String>,
        scope: Option<String>,
        revision: Option<String>,
    ) -> Self {
        Self {
            source: source.into(),
            scope,
            revision,
            acquired_at: Utc::now(),
        }
    }

    pub(crate) fn source(&self) -> &str {
        &self.source
    }

    pub(crate) fn scope(&self) -> Option<&str> {
        self.scope.as_deref()
    }

    pub(crate) fn revision(&self) -> Option<&str> {
        self.revision.as_deref()
    }

    /// When the content was acquired or, for remembered content, last
    /// confirmed to hold.
    pub(crate) fn acquired_at(&self) -> DateTime<Utc> {
        self.acquired_at
    }

    pub(crate) fn confirmed_at(mut self, at: DateTime<Utc>) -> Self {
        self.acquired_at = at;
        self
    }
}
