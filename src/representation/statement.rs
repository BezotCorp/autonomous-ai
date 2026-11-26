use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::representation::{notation_parser::NotationParser, term::Term};

/// Relation between terms, optionally negated. A negated statement is an
/// explicit proof of absence; a missing statement only means that no proof
/// is known. Statements containing variables act as patterns.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(remote = "Self")]
pub(crate) struct Statement {
    predicate: String,
    arguments: Vec<Term>,
    negated: bool,
}

impl Statement {
    pub(crate) fn new(predicate: impl Into<String>, arguments: Vec<Term>) -> Self {
        Self {
            predicate: predicate.into(),
            arguments,
            negated: false,
        }
    }

    pub(crate) fn negative(predicate: impl Into<String>, arguments: Vec<Term>) -> Self {
        Self {
            predicate: predicate.into(),
            arguments,
            negated: true,
        }
    }

    pub(crate) fn predicate(&self) -> &str {
        &self.predicate
    }

    pub(crate) fn arguments(&self) -> &[Term] {
        &self.arguments
    }

    pub(crate) fn is_negated(&self) -> bool {
        self.negated
    }

    pub(crate) fn complement(&self) -> Self {
        Self {
            predicate: self.predicate.clone(),
            arguments: self.arguments.clone(),
            negated: !self.negated,
        }
    }

    pub(crate) fn is_ground(&self) -> bool {
        !self.arguments.iter().any(Term::is_variable)
    }

    pub(crate) fn variables(&self) -> impl Iterator<Item = &str> {
        self.arguments.iter().filter_map(Term::variable)
    }

    /// Identifies the relation independently of its arguments, polarity included.
    pub(crate) fn signature(&self) -> String {
        format!("{}/{}", self.predicate, self.arguments.len())
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.negated {
            write!(formatter, "not ")?;
        }

        write!(formatter, "{}(", self.predicate)?;

        for (index, argument) in self.arguments.iter().enumerate() {
            if index > 0 {
                write!(formatter, ", ")?;
            }

            write!(formatter, "{argument}")?;
        }

        write!(formatter, ")")
    }
}

impl FromStr for Statement {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        NotationParser::new(value).statement_only()
    }
}

impl Serialize for Statement {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            Self::serialize(self, serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Statement {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            String::deserialize(deserializer)?
                .parse()
                .map_err(serde::de::Error::custom)
        } else {
            Self::deserialize(deserializer)
        }
    }
}
