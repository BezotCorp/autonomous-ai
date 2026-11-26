use serde::{Deserialize, Serialize};

use crate::memory::{
    activity_record::ActivityRecord, cost_statistic::CostStatistic,
    routing_statistic::RoutingStatistic,
};

/// Activities carried out on the intelligence's own initiative. From it
/// is learned when each kind of activity is worthwhile and what it costs.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct ActivityLog {
    records: Vec<ActivityRecord>,
}

impl ActivityLog {
    pub(crate) fn records(&self) -> &[ActivityRecord] {
        &self.records
    }

    pub(crate) fn record(&mut self, record: ActivityRecord) {
        self.records.push(record);
    }

    /// Outcomes of an activity focus in situations sharing a feature.
    pub(crate) fn statistic(&self, focus: &str, feature: &str) -> RoutingStatistic {
        let mut statistic = RoutingStatistic::default();

        for record in &self.records {
            if record.focus() == focus && record.features().iter().any(|known| known == feature) {
                statistic.observe(record.worthwhile());
            }
        }

        statistic
    }

    pub(crate) fn cost(&self, focus: &str) -> CostStatistic {
        let mut cost = CostStatistic::default();

        for record in self.records.iter().filter(|record| record.focus() == focus) {
            cost.observe((record.seconds() * 1e6) as u64);
        }

        cost
    }
}
