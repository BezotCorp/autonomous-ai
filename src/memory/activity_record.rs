use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::memory::activity::Activity;

/// One activity the intelligence chose to carry out on its own initiative,
/// the situation in which it chose it, and whether it proved worthwhile.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ActivityRecord {
    activity: Activity,
    focus: String,
    features: Vec<String>,
    started_at: DateTime<Utc>,
    seconds: f64,
    worthwhile: bool,
    summary: String,
}

impl ActivityRecord {
    pub(crate) fn new(
        activity: Activity,
        focus: String,
        features: Vec<String>,
        seconds: f64,
        worthwhile: bool,
        summary: String,
    ) -> Self {
        Self {
            activity,
            focus,
            features,
            started_at: Utc::now(),
            seconds,
            worthwhile,
            summary,
        }
    }

    pub(crate) fn activity(&self) -> Activity {
        self.activity
    }

    pub(crate) fn focus(&self) -> &str {
        &self.focus
    }

    pub(crate) fn features(&self) -> &[String] {
        &self.features
    }

    pub(crate) fn seconds(&self) -> f64 {
        self.seconds
    }

    pub(crate) fn worthwhile(&self) -> bool {
        self.worthwhile
    }

    pub(crate) fn summary(&self) -> &str {
        &self.summary
    }

    pub(crate) fn started_at(&self) -> DateTime<Utc> {
        self.started_at
    }
}
