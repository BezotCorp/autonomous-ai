use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Last known constraint of a consumable resource, as reported by the
/// resource itself (or imposed by an experiment). An unlimited resource
/// has no limit and never becomes scarce.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct ResourceState {
    limit: Option<f64>,
    remaining: Option<f64>,
    resets_at: Option<DateTime<Utc>>,
    observed_at: DateTime<Utc>,
}

impl ResourceState {
    pub(crate) fn limited(limit: f64, remaining: f64, resets_at: Option<DateTime<Utc>>) -> Self {
        Self {
            limit: Some(limit),
            remaining: Some(remaining.clamp(0.0, limit.max(0.0))),
            resets_at,
            observed_at: Utc::now(),
        }
    }

    /// Units available at the given instant, `None` when unlimited. Once
    /// the reset instant has passed, the whole limit is available again.
    pub(crate) fn available(&self, now: DateTime<Utc>) -> Option<f64> {
        let limit = self.limit?;

        if self.resets_at.is_some_and(|reset| reset <= now) {
            return Some(limit);
        }

        Some(self.remaining.unwrap_or(limit))
    }

    pub(crate) fn resets_at(&self) -> Option<DateTime<Utc>> {
        self.resets_at
    }

    /// Accounts for units consumed since the state was observed.
    pub(crate) fn consume(&mut self, units: f64, now: DateTime<Utc>) {
        if let Some(available) = self.available(now) {
            if self.resets_at.is_some_and(|reset| reset <= now) {
                self.resets_at = None;
            }

            self.remaining = Some((available - units).max(0.0));
        }
    }

    /// Scarcity cost, in utility units, of consuming `units` of a resource
    /// of which `available` units remain (`None`: unlimited).
    ///
    /// It is zero for an unlimited resource, grows with the share of what
    /// remains that the consumption would use (`ln((R + 1) / (R + 1 - q))`,
    /// about `q / R` while plentiful) and each unit beyond what remains
    /// costs one more. Only an observed exhaustion makes the consumption
    /// impossible: then the cost is infinite until the resource resets.
    pub(crate) fn cost(available: Option<f64>, units: f64) -> f64 {
        let Some(available) = available else {
            return 0.0;
        };

        if units <= 0.0 {
            return 0.0;
        }

        if available <= 0.0 {
            return f64::INFINITY;
        }

        let covered = units.min(available);

        ((available + 1.0) / (available + 1.0 - covered)).ln() + (units - covered)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::ResourceState;

    #[test]
    fn scarcity_grows_as_the_resource_runs_out() {
        let now = Utc::now();
        let plentiful = ResourceState::limited(5000.0, 5000.0, None);
        let scarce = ResourceState::limited(5000.0, 40.0, None);
        let exhausted = ResourceState::limited(5000.0, 0.0, Some(now + Duration::hours(1)));
        let reset = ResourceState::limited(5000.0, 0.0, Some(now - Duration::seconds(1)));

        let cost = |state: &ResourceState, units| ResourceState::cost(state.available(now), units);

        assert_eq!(ResourceState::cost(None, 1e6), 0.0);
        assert!(cost(&plentiful, 30.0) < 0.01);
        assert!(cost(&scarce, 30.0) > 1.0);
        assert!(cost(&scarce, 60.0) > cost(&scarce, 30.0) + 10.0);
        assert!(cost(&exhausted, 1.0).is_infinite());
        assert_eq!(cost(&exhausted, 0.0), 0.0);
        assert!(cost(&reset, 1.0) < 0.001);
    }
}
