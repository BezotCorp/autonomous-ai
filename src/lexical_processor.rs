use std::collections::BTreeSet;

use crate::{
    lexical_rule::LexicalRule, relation_fact::RelationFact, symbolic_dataset::SymbolicDataset,
};

pub(crate) struct LexicalProcessor;

impl LexicalProcessor {
    fn tokens(value: &str) -> Vec<String> {
        value
            .split(|character: char| !character.is_alphanumeric())
            .filter(|token| !token.is_empty())
            .map(str::to_lowercase)
            .collect()
    }

    fn contains_phrase(text: &[String], phrase: &[String]) -> bool {
        !phrase.is_empty() && text.windows(phrase.len()).any(|window| window == phrase)
    }

    fn matches(text: &str, rule: &LexicalRule) -> bool {
        let text = Self::tokens(text);

        rule.phrases()
            .iter()
            .any(|phrase| Self::contains_phrase(&text, &Self::tokens(phrase)))
    }

    pub(crate) fn derive(
        dataset: &SymbolicDataset,
        facts: &BTreeSet<RelationFact>,
    ) -> Vec<RelationFact> {
        let mut derived = BTreeSet::new();

        for fact in facts {
            if fact.predicate() != "documented_symbol" || fact.arguments().len() < 5 {
                continue;
            }

            let arguments = fact.arguments();

            let package = &arguments[0];
            let version = &arguments[1];
            let symbol = &arguments[2];
            let documentation = &arguments[4];

            if documentation.is_empty() || documentation == "Null" {
                continue;
            }

            for rule in dataset.lexical_rules() {
                if !Self::matches(documentation, rule) {
                    continue;
                }

                derived.insert(RelationFact::new(
                    "documented_capability",
                    vec![
                        package.clone(),
                        version.clone(),
                        symbol.clone(),
                        rule.capability().to_owned(),
                        rule.id().to_owned(),
                    ],
                ));
            }
        }

        derived.into_iter().collect()
    }
}

impl crate::cognitive_processor::CognitiveProcessor for LexicalProcessor {
    fn process(
        &self,
        dataset: &SymbolicDataset,
        facts: &BTreeSet<RelationFact>,
    ) -> Vec<RelationFact> {
        Self::derive(dataset, facts)
    }
}
