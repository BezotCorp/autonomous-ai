use serde::{Deserialize, Serialize};

use crate::representation::{document::Document, rule::Rule, statement::Statement};

/// What an individual transmits. It asserts statements (its own, or
/// relayed with the chain of their sources), may pass on rules it
/// conjectures or holds, may ask questions and may carry texts. Nothing
/// requires a question, nor an answer.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Message {
    statements: Vec<Statement>,
    relayed: Vec<(Statement, String)>,
    questions: Vec<Statement>,
    documents: Vec<Document>,
    rules: Vec<Rule>,
}

impl Message {
    pub(crate) fn new(
        statements: Vec<Statement>,
        relayed: Vec<(Statement, String)>,
        questions: Vec<Statement>,
        documents: Vec<Document>,
        rules: Vec<Rule>,
    ) -> Self {
        Self {
            statements,
            relayed,
            questions,
            documents,
            rules,
        }
    }

    /// Rules passed on: received as conjectures, never as established laws.
    pub(crate) fn rules(&self) -> &[Rule] {
        &self.rules
    }

    pub(crate) fn statements(&self) -> &[Statement] {
        &self.statements
    }

    /// Statements the sender had itself received, with where they came from.
    pub(crate) fn relayed(&self) -> &[(Statement, String)] {
        &self.relayed
    }

    pub(crate) fn questions(&self) -> &[Statement] {
        &self.questions
    }

    pub(crate) fn documents(&self) -> &[Document] {
        &self.documents
    }
}
