use serde::{Deserialize, Serialize};

/// Observed outcomes of a module in situations sharing one feature.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub(crate) struct RoutingStatistic {
    successes: u64,
    failures: u64,
}

impl RoutingStatistic {
    pub(crate) fn observe(&mut self, success: bool) {
        if success {
            self.successes += 1;
        } else {
            self.failures += 1;
        }
    }

    pub(crate) fn successes(&self) -> u64 {
        self.successes
    }

    pub(crate) fn failures(&self) -> u64 {
        self.failures
    }
}
