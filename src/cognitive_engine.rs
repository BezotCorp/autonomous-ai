use std::collections::BTreeSet;

use crate::cognition_result::CognitionResult;
use crate::cognitive_registry::CognitiveRegistry;
use crate::mathematical_processor::MathematicalProcessor;
use crate::relation_fact::RelationFact;
use crate::relational_engine::RelationalEngine;
use crate::symbolic_dataset::SymbolicDataset;

pub(crate) struct CognitiveEngine {
    registry: CognitiveRegistry,
    maximum_rounds: usize,
}

impl CognitiveEngine {
    pub(crate) fn new(registry: CognitiveRegistry, maximum_rounds: usize) -> Self {
        Self {
            registry,
            maximum_rounds,
        }
    }

    pub(crate) fn infer(
        &self,
        dataset: &SymbolicDataset,
        observed: Vec<RelationFact>,
    ) -> Result<CognitionResult, String> {
        let mut facts: BTreeSet<_> = observed.into_iter().collect();
        let mut deductions = BTreeSet::new();

        for _ in 0..self.maximum_rounds {
            let previous_count = facts.len();

            let derived: BTreeSet<_> = self
                .registry
                .processors()
                .flat_map(|processor| processor.process(dataset, &facts))
                .collect();

            facts.extend(derived);

            let (inferred, new_deductions) =
                RelationalEngine::infer(dataset.relational_rules(), facts)?;

            facts = inferred;
            deductions.extend(new_deductions);

            if facts.len() == previous_count {
                let evidence_scores = MathematicalProcessor::evaluate(dataset, &facts);

                return Ok(CognitionResult::new(
                    facts.into_iter().collect(),
                    deductions.into_iter().collect(),
                    evidence_scores,
                ));
            }
        }

        Err("Cognitive inference did not converge".into())
    }
}
