use std::collections::{BTreeMap, BTreeSet};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::memory::cached_response::CachedResponse;

/// Technical store of acquired data, shared by every acquisition module.
/// It is not knowledge: it only spares re-acquiring data whose identity
/// (content address or validator) shows it did not change. Its size is
/// bounded; the least recently used entries are dropped first.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct AcquisitionCache {
    entries: BTreeMap<String, CachedResponse>,
}

impl AcquisitionCache {
    const MAXIMUM_BYTES: usize = 64 * 1024 * 1024;

    pub(crate) fn get(&self, key: &str) -> Option<&CachedResponse> {
        self.entries.get(key)
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn bytes(&self) -> usize {
        self.entries.values().map(CachedResponse::size).sum()
    }

    /// Integrates what an episode stored and used, then enforces the bound.
    pub(crate) fn integrate(
        &mut self,
        stored: BTreeMap<String, CachedResponse>,
        used: BTreeSet<String>,
    ) {
        let now = Utc::now();

        self.entries.extend(stored);

        for key in used {
            if let Some(entry) = self.entries.get_mut(&key) {
                entry.mark_used(now);
            }
        }

        let mut bytes = self.bytes();

        if bytes <= Self::MAXIMUM_BYTES {
            return;
        }

        let mut by_age: Vec<(String, chrono::DateTime<Utc>, usize)> = self
            .entries
            .iter()
            .map(|(key, entry)| (key.clone(), entry.last_used_at(), entry.size()))
            .collect();

        by_age.sort_by_key(|(_, used, _)| *used);

        for (key, _, size) in by_age {
            if bytes <= Self::MAXIMUM_BYTES {
                break;
            }

            self.entries.remove(&key);
            bytes -= size;
        }
    }
}
