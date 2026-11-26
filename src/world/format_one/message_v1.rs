use serde::Deserialize;

use crate::{
    representation::{document::Document, statement::Statement},
    world::message::Message,
};

/// A message as format 1 of environments stored it (before rules could be
/// passed on). Read only, to migrate.
#[derive(Deserialize)]
pub(crate) struct MessageV1 {
    statements: Vec<Statement>,
    relayed: Vec<(Statement, String)>,
    questions: Vec<Statement>,
    documents: Vec<Document>,
}

impl MessageV1 {
    pub(crate) fn migrate(self) -> Message {
        Message::new(
            self.statements,
            self.relayed,
            self.questions,
            self.documents,
            Vec::new(),
        )
    }
}
