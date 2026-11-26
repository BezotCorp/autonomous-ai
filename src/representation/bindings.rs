use std::collections::BTreeMap;

use crate::representation::{statement::Statement, term::Term};

/// Substitution of variables by terms, produced by unification.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Bindings {
    values: BTreeMap<String, Term>,
}

impl Bindings {
    pub(crate) fn get(&self, variable: &str) -> Option<&Term> {
        self.values.get(variable)
    }

    /// Extends the bindings so that `pattern` becomes equal to `fact`.
    /// Polarity must match: a negated pattern only matches explicit negations.
    pub(crate) fn unify(&self, pattern: &Statement, fact: &Statement) -> Option<Self> {
        if pattern.predicate() != fact.predicate()
            || pattern.is_negated() != fact.is_negated()
            || pattern.arguments().len() != fact.arguments().len()
        {
            return None;
        }

        let mut bindings = self.clone();

        for (term, value) in pattern.arguments().iter().zip(fact.arguments()) {
            let term = match term.variable() {
                Some(name) => match bindings.values.get(name) {
                    Some(bound) => bound.clone(),
                    None => {
                        if !value.is_variable() {
                            bindings.values.insert(name.to_owned(), value.clone());
                        }
                        continue;
                    }
                },
                None => term.clone(),
            };

            if !value.is_variable() && &term != value {
                return None;
            }
        }

        Some(bindings)
    }

    /// Replaces every bound variable of the pattern by its value.
    pub(crate) fn apply(&self, pattern: &Statement) -> Statement {
        let arguments = pattern
            .arguments()
            .iter()
            .map(|term| {
                term.variable()
                    .and_then(|name| self.values.get(name))
                    .unwrap_or(term)
                    .clone()
            })
            .collect();

        if pattern.is_negated() {
            Statement::negative(pattern.predicate(), arguments)
        } else {
            Statement::new(pattern.predicate(), arguments)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Bindings;
    use crate::representation::statement::Statement;

    #[test]
    fn unifies_consistently() {
        let pattern: Statement = "parent(?x, ?y)".parse().unwrap();
        let fact: Statement = "parent(alice, bob)".parse().unwrap();
        let bindings = Bindings::default().unify(&pattern, &fact).unwrap();

        assert_eq!(
            bindings
                .apply(&"child(?y, ?x)".parse().unwrap())
                .to_string(),
            "child(bob, alice)"
        );
        assert!(
            bindings
                .unify(&"parent(?y, ?x)".parse().unwrap(), &fact)
                .is_none()
        );
        assert!(
            Bindings::default()
                .unify(&"not parent(?x, ?y)".parse().unwrap(), &fact)
                .is_none()
        );
    }
}
