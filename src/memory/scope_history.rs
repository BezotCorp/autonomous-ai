use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Successive verifications of one observed scope of a source (e.g. one
/// repository as read by one module): its latest revision, when it was
/// last confirmed, and how often it was found changed. From it follows a
/// learned rate of change, hence the probability that what was observed
/// no longer holds after some time.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ScopeHistory {
    revision: Option<String>,
    first_observed_at: DateTime<Utc>,
    last_verified_at: DateTime<Utc>,
    verifications: u64,
    changes: u64,
    exposure_seconds: f64,
}

impl ScopeHistory {
    pub(crate) fn new(revision: Option<String>, at: DateTime<Utc>) -> Self {
        Self {
            revision,
            first_observed_at: at,
            last_verified_at: at,
            verifications: 1,
            changes: 0,
            exposure_seconds: 0.0,
        }
    }

    /// Records a new observation of the scope. Returns whether the
    /// revision changed since the previous one.
    pub(crate) fn verify(&mut self, revision: Option<&str>, at: DateTime<Utc>) -> bool {
        let elapsed = (at - self.last_verified_at).num_milliseconds().max(0) as f64 / 1000.0;
        let changed = match (self.revision.as_deref(), revision) {
            (Some(previous), Some(current)) => previous != current,
            _ => false,
        };

        self.exposure_seconds += elapsed;
        self.verifications += 1;
        self.last_verified_at = self.last_verified_at.max(at);

        if changed {
            self.changes += 1;
        }

        if revision.is_some() {
            self.revision = revision.map(str::to_owned);
        }

        changed
    }

    pub(crate) fn revision(&self) -> Option<&str> {
        self.revision.as_deref()
    }

    pub(crate) fn last_verified_at(&self) -> DateTime<Utc> {
        self.last_verified_at
    }

    pub(crate) fn changes(&self) -> u64 {
        self.changes
    }

    pub(crate) fn verifications(&self) -> u64 {
        self.verifications
    }

    /// Estimated changes per second: observed changes over observed time,
    /// regularised by a prior (`prior_changes` over `prior_seconds`).
    pub(crate) fn change_rate(&self, prior_changes: f64, prior_seconds: f64) -> f64 {
        (self.changes as f64 + prior_changes) / (self.exposure_seconds + prior_seconds).max(1e-9)
    }

    /// Probability that the scope changed since its last verification,
    /// assuming changes occur independently at the learned rate.
    pub(crate) fn staleness(
        &self,
        now: DateTime<Utc>,
        prior_changes: f64,
        prior_seconds: f64,
    ) -> f64 {
        let age = (now - self.last_verified_at).num_milliseconds().max(0) as f64 / 1000.0;

        1.0 - (-self.change_rate(prior_changes, prior_seconds) * age).exp()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::ScopeHistory;

    #[test]
    fn learns_how_fast_a_scope_changes() {
        let start = Utc::now() - Duration::days(10);
        let day = 86_400.0;
        let mut stable = ScopeHistory::new(Some("a".to_owned()), start);
        let mut volatile = ScopeHistory::new(Some("a".to_owned()), start);

        for offset in 1..=9 {
            let at = start + Duration::days(offset);

            stable.verify(Some("a"), at);
            volatile.verify(Some(&offset.to_string()), at);
        }

        let now = start + Duration::days(10);

        assert_eq!(volatile.changes(), 9);
        assert!(stable.staleness(now, 1.0, day) < 0.15);
        assert!(volatile.staleness(now, 1.0, day) > 0.6);
        assert_eq!(stable.staleness(stable.last_verified_at(), 1.0, day), 0.0);
    }
}
