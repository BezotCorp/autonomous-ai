use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::latent::reaction_record::ReactionRecord;

/// Register of the occurrences met during the existence: for each
/// signature, how often it happened and what it changed in the latent
/// state; and which occurrences followed which (the observed relations
/// between actions and reactions). It grows with whatever the individual
/// meets: nothing in it is declared in advance.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct ReactionCatalog {
    reactions: BTreeMap<String, ReactionRecord>,
    sequences: BTreeMap<String, BTreeMap<String, u64>>,
    last: Option<String>,
}

impl ReactionCatalog {
    pub(crate) fn record(&mut self, signature: &str, impulse: &[f64], lived_seconds: f64) {
        self.reactions
            .entry(signature.to_owned())
            .or_insert_with(|| ReactionRecord::new(lived_seconds, impulse.len()))
            .observe(impulse, lived_seconds);

        if let Some(previous) = self.last.replace(signature.to_owned()) {
            *self
                .sequences
                .entry(previous)
                .or_default()
                .entry(signature.to_owned())
                .or_default() += 1;
        }
    }

    pub(crate) fn reactions(&self) -> &BTreeMap<String, ReactionRecord> {
        &self.reactions
    }

    pub(crate) fn sequences(&self) -> &BTreeMap<String, BTreeMap<String, u64>> {
        &self.sequences
    }
}
