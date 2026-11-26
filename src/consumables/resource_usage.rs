use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Consumption of external resources over some period: units consumed
/// and requests sent per resource, acquisitions avoided thanks to stored
/// or unchanged data, requests repeating an earlier one, and requests
/// refused beforehand because the resource could not afford them.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct ResourceUsage {
    consumption: BTreeMap<String, f64>,
    requests: BTreeMap<String, u64>,
    avoided: u64,
    redundant: u64,
    refused: u64,
}

impl ResourceUsage {
    pub(crate) fn record_request(&mut self, resource: &str, units: f64, redundant: bool) {
        *self.consumption.entry(resource.to_owned()).or_default() += units;
        *self.requests.entry(resource.to_owned()).or_default() += 1;

        if redundant {
            self.redundant += 1;
        }
    }

    pub(crate) fn record_avoided(&mut self) {
        self.avoided += 1;
    }

    pub(crate) fn record_refused(&mut self) {
        self.refused += 1;
    }

    pub(crate) fn add(&mut self, other: &Self) {
        for (resource, units) in &other.consumption {
            *self.consumption.entry(resource.clone()).or_default() += units;
        }

        for (resource, requests) in &other.requests {
            *self.requests.entry(resource.clone()).or_default() += requests;
        }

        self.avoided += other.avoided;
        self.redundant += other.redundant;
        self.refused += other.refused;
    }

    /// What was used since an earlier snapshot of the same counters.
    pub(crate) fn since(&self, earlier: &Self) -> Self {
        let mut difference = Self::default();

        for (resource, units) in &self.consumption {
            let before = earlier.consumption.get(resource).copied().unwrap_or(0.0);

            if *units - before != 0.0 {
                difference
                    .consumption
                    .insert(resource.clone(), *units - before);
            }
        }

        for (resource, requests) in &self.requests {
            let before = earlier.requests.get(resource).copied().unwrap_or(0);

            if *requests > before {
                difference
                    .requests
                    .insert(resource.clone(), *requests - before);
            }
        }

        difference.avoided = self.avoided - earlier.avoided;
        difference.redundant = self.redundant - earlier.redundant;
        difference.refused = self.refused - earlier.refused;
        difference
    }

    pub(crate) fn consumption(&self) -> &BTreeMap<String, f64> {
        &self.consumption
    }

    pub(crate) fn requests(&self) -> &BTreeMap<String, u64> {
        &self.requests
    }

    pub(crate) fn avoided(&self) -> u64 {
        self.avoided
    }

    pub(crate) fn redundant(&self) -> u64 {
        self.redundant
    }

    pub(crate) fn refused(&self) -> u64 {
        self.refused
    }
}
