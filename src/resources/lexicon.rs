use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::resources::lexical_entry::LexicalEntry;

/// Linguistic knowledge associating word forms with lemmas, categories,
/// features and senses. Several entries may share a form: ambiguity is
/// preserved and left to the reasoning.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Lexicon {
    #[serde(default)]
    sources: Vec<String>,
    entries: Vec<LexicalEntry>,
    #[serde(skip)]
    index: BTreeMap<String, Vec<usize>>,
}

impl Lexicon {
    pub(crate) fn index(mut self) -> Self {
        self.index.clear();

        for (position, entry) in self.entries.iter().enumerate() {
            self.index
                .entry(entry.form().to_lowercase())
                .or_default()
                .push(position);
        }

        self
    }

    pub(crate) fn lookup(&self, form: &str) -> impl Iterator<Item = &LexicalEntry> {
        self.index
            .get(&form.to_lowercase())
            .into_iter()
            .flatten()
            .map(|position| &self.entries[*position])
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn sources(&self) -> &[String] {
        &self.sources
    }
}
