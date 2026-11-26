use std::fmt;

use serde::{Deserialize, Serialize};

use crate::mathematics::{expression::Expression, rational::Rational};

/// Elementary value carried by a statement argument.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Term {
    Symbol(String),
    Text(String),
    Number(Rational),
    Version(semver::Version),
    Expression(Expression),
    Variable(String),
}

impl Term {
    pub(crate) fn symbol(value: impl Into<String>) -> Self {
        Self::Symbol(value.into())
    }

    pub(crate) fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    pub(crate) fn variable(&self) -> Option<&str> {
        match self {
            Self::Variable(name) => Some(name),
            _ => None,
        }
    }

    pub(crate) fn is_variable(&self) -> bool {
        matches!(self, Self::Variable(_))
    }
}

impl fmt::Display for Term {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Symbol(value) => write!(formatter, "{value}"),
            Self::Text(value) => write!(formatter, "{value:?}"),
            Self::Number(value) => write!(formatter, "{value}"),
            Self::Version(value) => write!(formatter, "v{value}"),
            Self::Expression(value) => write!(formatter, "[{value}]"),
            Self::Variable(name) => write!(formatter, "?{name}"),
        }
    }
}
