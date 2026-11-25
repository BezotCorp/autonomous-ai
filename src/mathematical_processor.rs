use std::collections::{BTreeMap, BTreeSet};

use crate::relation_fact::RelationFact;
use crate::symbolic_dataset::SymbolicDataset;
use crate::weighted_evidence::WeightedEvidence;

pub(crate) struct MathematicalProcessor;

impl MathematicalProcessor {
    pub(crate) fn evaluate(
        dataset: &SymbolicDataset,
        facts: &BTreeSet<RelationFact>,
    ) -> Vec<WeightedEvidence> {
        let mut evidence: BTreeMap<Vec<String>, BTreeMap<RelationFact, f64>> = BTreeMap::new();

        for weight in dataset.evidence_weights() {
            for fact in facts {
                if fact.predicate() != weight.predicate() {
                    continue;
                }

                let Some(subject) = weight
                    .subject_indices()
                    .iter()
                    .map(|index| fact.arguments().get(*index).cloned())
                    .collect::<Option<Vec<_>>>()
                else {
                    continue;
                };

                evidence
                    .entry(subject)
                    .or_default()
                    .entry(fact.clone())
                    .and_modify(|value| {
                        *value += weight.weight();
                    })
                    .or_insert(weight.weight());
            }
        }

        evidence
            .into_iter()
            .filter_map(|(subject, observations)| {
                let score: f64 = observations.values().sum();

                if !score.is_finite() {
                    return None;
                }

                Some(WeightedEvidence::new(
                    subject,
                    score,
                    observations.into_keys().collect(),
                ))
            })
            .collect()
    }
}
