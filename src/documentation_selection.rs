use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

#[derive(Default, Debug, Serialize)]
pub(crate) struct DocumentationSelection {
    rows: BTreeMap<String, Vec<BTreeMap<String, String>>>,
    unresolved: BTreeSet<String>,
    failures: Vec<String>,
    truncated: bool,
}

impl DocumentationSelection {
    pub(crate) fn insert(&mut self, query: String, rows: Vec<BTreeMap<String, String>>) {
        self.rows.insert(query, rows);
    }

    pub(crate) fn unresolved(&mut self, symbol: String) {
        self.unresolved.insert(symbol);
    }

    pub(crate) fn fail(&mut self, failure: impl Into<String>) {
        self.failures.push(failure.into());
    }

    pub(crate) fn truncate(&mut self) {
        self.truncated = true;
    }

    pub(crate) fn is_incomplete(&self) -> bool {
        self.truncated
            || self.rows.values().all(Vec::is_empty)
            || !self.unresolved.is_empty()
            || !self.failures.is_empty()
    }
}
