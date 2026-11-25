use std::collections::BTreeSet;

use crate::{symbolic_dataset::SymbolicDataset, symbolic_deduction::SymbolicDeduction};

pub(crate) struct SymbolicEngine;

impl SymbolicEngine {
    pub(crate) fn infer(
        dataset: &SymbolicDataset,
        initial_facts: &BTreeSet<String>,
    ) -> Vec<SymbolicDeduction> {
        let mut facts = initial_facts.clone();
        let mut applied = BTreeSet::new();
        let mut deductions = Vec::new();

        loop {
            let mut progressed = false;

            for rule in dataset.rules() {
                if applied.contains(rule.id()) {
                    continue;
                }

                if !rule.requires().iter().all(|fact| facts.contains(fact)) {
                    continue;
                }

                applied.insert(rule.id().to_owned());
                facts.insert(rule.concludes().to_owned());

                deductions.push(SymbolicDeduction::new(rule));

                progressed = true;
            }

            if !progressed {
                break;
            }
        }

        deductions
    }
}
