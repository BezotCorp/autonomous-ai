use serde::{Deserialize, Serialize};

use crate::memory::{
    hypothesis_record::HypothesisRecord, invocation_outcome::InvocationOutcome,
    knowledge_entry::KnowledgeEntry,
};

/// State of the memory a correction replaced, kept to revert it.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PreviousState {
    Hypothesis(Option<HypothesisRecord>),
    Fact(Option<KnowledgeEntry>),
    Outcome(Option<InvocationOutcome>),
}
