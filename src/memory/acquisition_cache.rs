use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::memory::cached_response::CachedResponse;

/// Technical store of acquired data, shared by every acquisition module.
/// It is not knowledge: it only spares re-acquiring data whose identity
/// (content address or validator) shows it did not change.
///
/// It is also the record of what the environment answered: the successive
/// versions of mutable data are kept with their date, so that a past
/// acquisition can be replayed exactly as it was, without reaching the
/// environment. Its size is bounded; the least recently used entries are
/// dropped first.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct AcquisitionCache {
    entries: BTreeMap<String, CachedResponse>,
    history: BTreeMap<String, Vec<CachedResponse>>,
}

impl AcquisitionCache {
    const MAXIMUM_BYTES: usize = 64 * 1024 * 1024;
    const MAXIMUM_VERSIONS: usize = 32;

    pub(crate) fn get(&self, key: &str) -> Option<&CachedResponse> {
        self.entries.get(key)
    }

    /// What was known of the data at an instant: immutable content
    /// whenever it was acquired, the version of mutable content that was
    /// current then. `None` when nothing was recorded by then.
    pub(crate) fn at(&self, key: &str, instant: DateTime<Utc>) -> Option<&CachedResponse> {
        let latest = self.entries.get(key)?;

        if latest.is_immutable() {
            return Some(latest);
        }

        self.history
            .get(key)
            .into_iter()
            .flatten()
            .chain([latest])
            .filter(|version| version.stored_at() <= instant)
            .max_by_key(|version| version.stored_at())
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn bytes(&self) -> usize {
        self.entries
            .values()
            .map(CachedResponse::size)
            .sum::<usize>()
            + self
                .history
                .values()
                .flatten()
                .map(CachedResponse::size)
                .sum::<usize>()
    }

    /// Integrates what an episode stored and used, then enforces the bound.
    pub(crate) fn integrate(
        &mut self,
        stored: BTreeMap<String, CachedResponse>,
        used: BTreeSet<String>,
    ) {
        let now = Utc::now();

        for (key, response) in stored {
            // Unchanged data was known since it was first stored: keep that
            // date, so that replays know what was available when.
            if let Some(previous) = self.entries.get(&key)
                && previous.body() == response.body()
            {
                let mut kept = previous.clone();

                kept.revalidate(response.validator().map(str::to_owned));
                self.entries.insert(key, kept);
                continue;
            }

            if let Some(previous) = self.entries.get(&key)
                && !response.is_immutable()
            {
                let versions = self.history.entry(key.clone()).or_default();

                versions.push(previous.clone());

                if versions.len() > Self::MAXIMUM_VERSIONS {
                    versions.remove(0);
                }
            }

            self.entries.insert(key, response);
        }

        for key in used {
            if let Some(entry) = self.entries.get_mut(&key) {
                entry.mark_used(now);
            }
        }

        let mut bytes = self.bytes();

        if bytes <= Self::MAXIMUM_BYTES {
            return;
        }

        let mut by_age: Vec<(String, DateTime<Utc>)> = self
            .entries
            .iter()
            .map(|(key, entry)| (key.clone(), entry.last_used_at()))
            .collect();

        by_age.sort_by_key(|(_, used)| *used);

        for (key, _) in by_age {
            if bytes <= Self::MAXIMUM_BYTES {
                break;
            }

            if let Some(entry) = self.entries.remove(&key) {
                bytes -= entry.size();
            }

            for version in self.history.remove(&key).into_iter().flatten() {
                bytes -= version.size();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use chrono::Utc;

    use super::AcquisitionCache;
    use crate::memory::cached_response::CachedResponse;

    #[test]
    fn keeps_dated_versions_of_mutable_data() {
        let mut cache = AcquisitionCache::default();
        let before = Utc::now();

        cache.integrate(
            BTreeMap::from([
                (
                    "list".to_owned(),
                    CachedResponse::mutable("v1".to_owned(), None),
                ),
                ("blob".to_owned(), CachedResponse::immutable("x".to_owned())),
            ]),
            BTreeSet::new(),
        );

        let between = Utc::now();

        std::thread::sleep(std::time::Duration::from_millis(2));
        cache.integrate(
            BTreeMap::from([(
                "list".to_owned(),
                CachedResponse::mutable("v2".to_owned(), None),
            )]),
            BTreeSet::new(),
        );

        assert!(cache.at("list", before).is_none());
        assert_eq!(cache.at("list", between).unwrap().body(), "v1");
        assert_eq!(cache.at("list", Utc::now()).unwrap().body(), "v2");
        assert_eq!(cache.at("blob", before).unwrap().body(), "x");
    }
}
