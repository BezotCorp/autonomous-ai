use std::collections::BTreeSet;

use crate::{relation_fact::RelationFact, symbolic_dataset::SymbolicDataset};

pub(crate) trait CognitiveProcessor {
    fn process(
        &self,
        dataset: &SymbolicDataset,
        facts: &BTreeSet<RelationFact>,
    ) -> Vec<RelationFact>;
}
