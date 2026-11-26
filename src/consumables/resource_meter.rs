use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Mutex, MutexGuard},
};

use chrono::Utc;

use crate::consumables::{resource_state::ResourceState, resource_usage::ResourceUsage};

/// Shared account of the external resources during one episode: the
/// constraints the resources reported, the budgets imposed on them, and
/// everything consumed. Acquisition code reports each request to it and
/// asks it beforehand whether the resource can afford one; the engine
/// reads it to anticipate and attribute costs.
///
/// It only enforces physical constraints: a request is refused when the
/// resource is known to be exhausted, never because it would be expensive.
#[derive(Debug, Default)]
pub(crate) struct ResourceMeter {
    ledger: Mutex<Ledger>,
}

#[derive(Debug, Default)]
struct Ledger {
    states: BTreeMap<String, ResourceState>,
    budgets: BTreeMap<String, f64>,
    usage: ResourceUsage,
    requested: BTreeSet<String>,
}

impl ResourceMeter {
    pub(crate) fn new(
        states: BTreeMap<String, ResourceState>,
        budgets: BTreeMap<String, f64>,
    ) -> Self {
        Self {
            ledger: Mutex::new(Ledger {
                states,
                budgets,
                ..Ledger::default()
            }),
        }
    }

    fn ledger(&self) -> MutexGuard<'_, Ledger> {
        // A panic while holding the lock cannot leave the counters in an
        // inconsistent state worth refusing: keep using them.
        self.ledger
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Checks that the resource can afford a consumption before trying it.
    pub(crate) fn authorize(&self, resource: &str, units: f64) -> Result<(), String> {
        let mut ledger = self.ledger();

        match ledger.available(resource) {
            Some(available) if available < units => {
                ledger.usage.record_refused();

                let reset = ledger
                    .states
                    .get(resource)
                    .and_then(ResourceState::resets_at)
                    .map(|reset| format!(", resets at {reset}"))
                    .unwrap_or_default();

                Err(format!(
                    "resource {resource} exhausted: {available} available, {units} required{reset}"
                ))
            }
            _ => Ok(()),
        }
    }

    /// Records a request actually sent. `key` identifies what was asked so
    /// that repeating the same request within the episode is detected.
    pub(crate) fn record(&self, resource: &str, key: &str, units: f64) {
        let mut ledger = self.ledger();
        let now = Utc::now();
        let redundant = !ledger.requested.insert(key.to_owned());

        ledger.usage.record_request(resource, units, redundant);

        if let Some(state) = ledger.states.get_mut(resource) {
            state.consume(units, now);
        }

        if let Some(budget) = ledger.budgets.get_mut(resource) {
            *budget = (*budget - units).max(0.0);
        }
    }

    /// Records an acquisition made unnecessary by stored or unchanged data.
    pub(crate) fn avoid(&self) {
        self.ledger().usage.record_avoided();
    }

    /// Replaces the known constraint of a resource by what it reported.
    pub(crate) fn observe(&self, resource: &str, state: ResourceState) {
        self.ledger().states.insert(resource.to_owned(), state);
    }

    pub(crate) fn usage(&self) -> ResourceUsage {
        self.ledger().usage.clone()
    }

    pub(crate) fn states(&self) -> BTreeMap<String, ResourceState> {
        self.ledger().states.clone()
    }

    /// Scarcity cost of consuming these units of each resource now.
    pub(crate) fn scarcity_cost<'a>(
        &self,
        consumption: impl IntoIterator<Item = (&'a String, &'a f64)>,
    ) -> f64 {
        let ledger = self.ledger();

        consumption
            .into_iter()
            .map(|(resource, units)| ResourceState::cost(ledger.available(resource), *units))
            .sum()
    }

    /// Units available of every resource with a known constraint.
    pub(crate) fn availabilities(&self) -> BTreeMap<String, Option<f64>> {
        let ledger = self.ledger();

        ledger
            .states
            .keys()
            .chain(ledger.budgets.keys())
            .map(|resource| (resource.clone(), ledger.available(resource)))
            .collect()
    }

    /// Scarcity cost of a consumption given the availabilities before it.
    pub(crate) fn scarcity_cost_given(
        availabilities: &BTreeMap<String, Option<f64>>,
        consumption: &BTreeMap<String, f64>,
    ) -> f64 {
        consumption
            .iter()
            .map(|(resource, units)| {
                ResourceState::cost(availabilities.get(resource).copied().flatten(), *units)
            })
            .sum()
    }

    /// Among alternative ways of acquiring the same data, each described by
    /// the units it would consume and the number of requests it would send,
    /// the one with the lowest scarcity cost; fewer requests break ties.
    pub(crate) fn cheapest(&self, alternatives: &[(Vec<(String, f64)>, usize)]) -> Option<usize> {
        let ledger = self.ledger();

        alternatives
            .iter()
            .enumerate()
            .map(|(index, (consumption, requests))| {
                let cost: f64 = consumption
                    .iter()
                    .map(|(resource, units)| {
                        ResourceState::cost(ledger.available(resource), *units)
                    })
                    .sum();

                (index, cost, *requests)
            })
            .min_by(|left, right| left.1.total_cmp(&right.1).then(left.2.cmp(&right.2)))
            .map(|(index, _, _)| index)
    }
}

impl Ledger {
    fn available(&self, resource: &str) -> Option<f64> {
        let reported = self
            .states
            .get(resource)
            .and_then(|state| state.available(Utc::now()));
        let budget = self.budgets.get(resource).copied();

        match (reported, budget) {
            (Some(reported), Some(budget)) => Some(reported.min(budget)),
            (reported, budget) => reported.or(budget),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::ResourceMeter;
    use crate::consumables::resource_state::ResourceState;

    #[test]
    fn accounts_requests_budgets_and_alternatives() {
        let meter = ResourceMeter::new(
            BTreeMap::from([(
                "api:rest".to_owned(),
                ResourceState::limited(100.0, 3.0, None),
            )]),
            BTreeMap::from([("api:query".to_owned(), 50.0)]),
        );

        assert!(meter.authorize("api:rest", 3.0).is_ok());
        meter.record("api:rest", "a", 1.0);
        meter.record("api:rest", "a", 1.0);
        assert!(meter.authorize("api:rest", 2.0).is_err());
        assert_eq!(meter.availabilities().get("api:rest"), Some(&Some(1.0)));
        assert!(!meter.availabilities().contains_key("other"));

        let alternatives = vec![
            (vec![("api:rest".to_owned(), 1.0)], 1),
            (vec![("api:query".to_owned(), 1.0)], 1),
        ];

        assert_eq!(meter.cheapest(&alternatives), Some(1));

        let usage = meter.usage();

        assert_eq!(usage.requests().get("api:rest"), Some(&2));
        assert_eq!(usage.redundant(), 1);
        assert_eq!(usage.refused(), 1);
    }
}
