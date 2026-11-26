use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::resources::sense::Sense;

/// Inflected form of a word with its lemma, grammatical category,
/// morphological features (Universal Dependencies conventions, e.g.
/// `Polarity=Neg`) and possible senses.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct LexicalEntry {
    form: String,
    lemma: String,
    category: String,
    #[serde(default)]
    features: BTreeMap<String, String>,
    #[serde(default)]
    senses: Vec<Sense>,
}

impl LexicalEntry {
    pub(crate) fn form(&self) -> &str {
        &self.form
    }

    pub(crate) fn lemma(&self) -> &str {
        &self.lemma
    }

    pub(crate) fn category(&self) -> &str {
        &self.category
    }

    pub(crate) fn features(&self) -> &BTreeMap<String, String> {
        &self.features
    }

    pub(crate) fn senses(&self) -> &[Sense] {
        &self.senses
    }
}
