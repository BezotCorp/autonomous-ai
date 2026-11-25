use std::collections::{BTreeMap, BTreeSet};

use crate::{
    relation_fact::RelationFact, relation_pattern::RelationPattern,
    relational_deduction::RelationalDeduction, relational_rule::RelationalRule,
};

type Bindings = BTreeMap<String, String>;

use crate::rule_match::RuleMatch;

pub(crate) struct RelationalEngine;

impl RelationalEngine {
    const MAX_PASSES: usize = 64;
    const MAX_FACTS: usize = 50_000;
    const MAX_MATCHES: usize = 16_384;

    pub(crate) fn infer(
        rules: &[RelationalRule],
        mut facts: BTreeSet<RelationFact>,
    ) -> Result<(BTreeSet<RelationFact>, Vec<RelationalDeduction>), String> {
        let mut deductions = BTreeSet::new();

        for rule in rules {
            rule.validate()?;
        }

        for _ in 0..Self::MAX_PASSES {
            let previous_count = facts.len();

            for rule in rules {
                let matches = Self::match_rule(rule, &facts)?;

                for matched in matches {
                    let conclusion = Self::instantiate(rule.concludes(), matched.bindings())?;

                    facts.insert(conclusion.clone());

                    deductions.insert(RelationalDeduction::new(
                        rule.id().to_owned(),
                        matched.into_premises(),
                        conclusion,
                        rule.explanation().to_owned(),
                    ));

                    if facts.len() > Self::MAX_FACTS {
                        return Err("Relational fact limit exceeded".into());
                    }
                }
            }

            if facts.len() == previous_count {
                return Ok((facts, deductions.into_iter().collect()));
            }
        }

        Err("Relational inference did not converge within the pass limit".into())
    }

    fn match_rule(
        rule: &RelationalRule,
        facts: &BTreeSet<RelationFact>,
    ) -> Result<Vec<RuleMatch>, String> {
        let mut results = Vec::new();

        Self::match_premises(
            rule.requires(),
            facts,
            0,
            Bindings::new(),
            Vec::new(),
            &mut results,
        )?;

        Ok(results)
    }

    fn match_premises(
        patterns: &[RelationPattern],
        facts: &BTreeSet<RelationFact>,
        index: usize,
        bindings: Bindings,
        premises: Vec<RelationFact>,
        results: &mut Vec<RuleMatch>,
    ) -> Result<(), String> {
        if index == patterns.len() {
            if results.len() >= Self::MAX_MATCHES {
                return Err("Relational match limit exceeded".into());
            }

            results.push(RuleMatch::new(bindings, premises));

            return Ok(());
        }

        let pattern = &patterns[index];

        for fact in facts {
            let Some(next_bindings) = Self::unify(pattern, fact, &bindings) else {
                continue;
            };

            let mut next_premises = premises.clone();
            next_premises.push(fact.clone());

            Self::match_premises(
                patterns,
                facts,
                index + 1,
                next_bindings,
                next_premises,
                results,
            )?;
        }

        Ok(())
    }

    fn unify(
        pattern: &RelationPattern,
        fact: &RelationFact,
        bindings: &Bindings,
    ) -> Option<Bindings> {
        if pattern.predicate() != fact.predicate()
            || pattern.arguments().len() != fact.arguments().len()
        {
            return None;
        }

        let mut bindings = bindings.clone();

        for (term, value) in pattern.arguments().iter().zip(fact.arguments()) {
            if let Some(variable) = term.strip_prefix('?') {
                match bindings.get(variable) {
                    Some(existing) if existing != value => {
                        return None;
                    }
                    Some(_) => {}
                    None => {
                        bindings.insert(variable.to_owned(), value.clone());
                    }
                }
            } else if term != value {
                return None;
            }
        }

        Some(bindings)
    }

    fn instantiate(pattern: &RelationPattern, bindings: &Bindings) -> Result<RelationFact, String> {
        let arguments = pattern
            .arguments()
            .iter()
            .map(|argument| {
                if let Some(variable) = argument.strip_prefix('?') {
                    bindings
                        .get(variable)
                        .cloned()
                        .ok_or_else(|| format!("Unbound relational variable: {variable}"))
                } else {
                    Ok(argument.clone())
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(RelationFact::new(pattern.predicate(), arguments))
    }
}
