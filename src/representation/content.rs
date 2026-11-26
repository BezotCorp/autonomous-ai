use std::fmt;

use serde::{Deserialize, Serialize};

use crate::representation::{
    content_kind::ContentKind, document::Document, rule::Rule, statement::Statement,
};

/// Unit of information held in the reasoning graph.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Content {
    /// Something asserted about the world.
    Statement(Statement),
    /// An implication usable by deduction.
    Rule(Rule),
    /// A statement pattern the intelligence lacks and would like to know.
    Need(Statement),
    /// Raw text awaiting interpretation.
    Document(Document),
}

impl Content {
    pub(crate) fn kind(&self) -> ContentKind {
        match self {
            Self::Statement(_) => ContentKind::Statement,
            Self::Rule(_) => ContentKind::Rule,
            Self::Need(_) => ContentKind::Need,
            Self::Document(_) => ContentKind::Document,
        }
    }

    pub(crate) fn statement(&self) -> Option<&Statement> {
        match self {
            Self::Statement(statement) => Some(statement),
            _ => None,
        }
    }

    pub(crate) fn rule(&self) -> Option<&Rule> {
        match self {
            Self::Rule(rule) => Some(rule),
            _ => None,
        }
    }

    pub(crate) fn need(&self) -> Option<&Statement> {
        match self {
            Self::Need(pattern) => Some(pattern),
            _ => None,
        }
    }

    pub(crate) fn document(&self) -> Option<&Document> {
        match self {
            Self::Document(document) => Some(document),
            _ => None,
        }
    }

    /// Short description used to characterise a situation (routing features).
    pub(crate) fn feature(&self) -> String {
        match self {
            Self::Statement(statement) => format!("statement:{}", statement.signature()),
            Self::Rule(rule) => format!("rule:{}", rule.conclusion().signature()),
            Self::Need(pattern) => format!("need:{}", pattern.signature()),
            Self::Document(_) => "document".to_owned(),
        }
    }
}

impl fmt::Display for Content {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Statement(statement) => write!(formatter, "{statement}"),
            Self::Rule(rule) => write!(formatter, "{rule}"),
            Self::Need(pattern) => write!(formatter, "need {pattern}"),
            Self::Document(document) => write!(formatter, "document {}", document.identifier()),
        }
    }
}
