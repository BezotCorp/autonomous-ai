use std::{collections::BTreeSet, fmt};

use serde::{Deserialize, Serialize};

use crate::representation::{rule::Rule, statement::Statement};

/// What the unconscious brings into the context of the conscious. Contents
/// enter the reasoning as defeasible elements; preferences only change how
/// a decision is weighed, never impose it.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InfluenceKind {
    /// A conjectured rule: hypothesis, association or new representation
    /// of a problem.
    Conjecture { rule: Rule },
    /// Something believed, brought to attention.
    Belief { statement: Statement },
    /// Information that may turn out to be needed.
    PotentialNeed { pattern: Statement },
    /// Something believed that deserves to be verified.
    Doubt { statement: Statement },
    /// A capability worth an actual chance in this kind of situation.
    Favour { module: String },
    /// A habitual choice worth questioning in this kind of situation.
    Question { module: String },
}

impl InfluenceKind {
    /// Predicates the influence is about, to recognise relevant situations.
    pub(crate) fn predicates(&self) -> BTreeSet<String> {
        let of = |statement: &Statement| statement.predicate().to_owned();

        match self {
            Self::Conjecture { rule } => rule
                .premises()
                .iter()
                .chain([rule.conclusion()])
                .map(of)
                .collect(),
            Self::Belief { statement } | Self::Doubt { statement } => {
                BTreeSet::from([of(statement)])
            }
            Self::PotentialNeed { pattern } => BTreeSet::from([of(pattern)]),
            Self::Favour { .. } | Self::Question { .. } => BTreeSet::new(),
        }
    }

    /// Module whose selection the influence weighs, if any, and whether it
    /// favours (`true`) or questions it.
    pub(crate) fn preference(&self) -> Option<(&str, bool)> {
        match self {
            Self::Favour { module } => Some((module, true)),
            Self::Question { module } => Some((module, false)),
            _ => None,
        }
    }
}

impl fmt::Display for InfluenceKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conjecture { rule } => write!(formatter, "conjecture {rule}"),
            Self::Belief { statement } => write!(formatter, "belief {statement}"),
            Self::PotentialNeed { pattern } => write!(formatter, "potential need {pattern}"),
            Self::Doubt { statement } => write!(formatter, "doubt {statement}"),
            Self::Favour { module } => write!(formatter, "favour {module}"),
            Self::Question { module } => write!(formatter, "question {module}"),
        }
    }
}
