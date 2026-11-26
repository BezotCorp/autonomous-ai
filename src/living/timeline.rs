use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::living::{life_event::LifeEvent, timeline_entry::TimelineEntry};

/// Chronological record of an existence. Recording never feeds back into
/// the dynamics: the timeline is written, never read, by the mechanisms
/// of the individual. Its length is bounded only against a runaway; were
/// the bound reached, the oldest entries would be dropped and counted by
/// kind.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Timeline {
    entries: VecDeque<TimelineEntry>,
    dropped: BTreeMap<String, u64>,
    totals: BTreeMap<String, u64>,
}

impl Timeline {
    /// Bound against a runaway, not a retention policy: an existence keeps
    /// its whole history (about 160 000 entries per 105 virtual days in
    /// practice). Reports select periods instead of truncating history.
    const MAXIMUM_ENTRIES: usize = 50_000_000;

    pub(crate) fn record(&mut self, lived: f64, event: LifeEvent) {
        *self.totals.entry(event.kind().to_owned()).or_default() += 1;
        self.entries.push_back(TimelineEntry::new(lived, event));

        while self.entries.len() > Self::MAXIMUM_ENTRIES {
            if let Some(entry) = self.entries.pop_front() {
                *self
                    .dropped
                    .entry(entry.event().kind().to_owned())
                    .or_default() += 1;
            }
        }
    }

    pub(crate) fn entries(&self) -> &VecDeque<TimelineEntry> {
        &self.entries
    }

    pub(crate) fn dropped(&self) -> &BTreeMap<String, u64> {
        &self.dropped
    }

    pub(crate) fn totals(&self) -> &BTreeMap<String, u64> {
        &self.totals
    }
}
