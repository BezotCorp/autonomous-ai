use serde::{Deserialize, Serialize};

use crate::representation::{document::Document, statement::Statement};

/// What an individual transmits. It asserts statements (its own, or
/// relayed with the chain of their sources), may ask questions and may
/// carry texts. Nothing requires a question, nor an answer.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Message {
    statements: Vec<Statement>,
    relayed: Vec<(Statement, String)>,
    questions: Vec<Statement>,
    documents: Vec<Document>,
}

impl Message {
    pub(crate) fn new(
        statements: Vec<Statement>,
        relayed: Vec<(Statement, String)>,
        questions: Vec<Statement>,
        documents: Vec<Document>,
    ) -> Self {
        Self {
            statements,
            relayed,
            questions,
            documents,
        }
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
