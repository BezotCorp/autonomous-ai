use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Mutex,
};

use crate::memory::{acquisition_cache::AcquisitionCache, cached_response::CachedResponse};

/// Episode view of the acquisition cache: reads the persistent store and
/// collects what the episode adds, to be integrated once it is over.
#[derive(Debug)]
pub(crate) struct ResponseStore<'a> {
    persistent: &'a AcquisitionCache,
    added: Mutex<(BTreeMap<String, CachedResponse>, BTreeSet<String>)>,
}

impl<'a> ResponseStore<'a> {
    pub(crate) fn new(persistent: &'a AcquisitionCache) -> Self {
        Self {
            persistent,
            added: Mutex::default(),
        }
    }

    pub(crate) fn get(&self, key: &str) -> Option<CachedResponse> {
        let mut added = self
            .added
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let found = added
            .0
            .get(key)
            .cloned()
            .or_else(|| self.persistent.get(key).cloned());

        if found.is_some() {
            added.1.insert(key.to_owned());
        }

        found
    }

    /// Data as recorded at an instant, for replays.
    pub(crate) fn recorded(
        &self,
        key: &str,
        instant: chrono::DateTime<chrono::Utc>,
    ) -> Option<CachedResponse> {
        self.persistent.at(key, instant).cloned()
    }

    pub(crate) fn put(&self, key: &str, response: CachedResponse) {
        self.added
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .0
            .insert(key.to_owned(), response);
    }

    /// Entries stored and keys used during the episode.
    pub(crate) fn into_parts(self) -> (BTreeMap<String, CachedResponse>, BTreeSet<String>) {
        self.added
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
