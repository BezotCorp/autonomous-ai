use serde::{Deserialize, Serialize};

use crate::representation::statement::Statement;

/// A capability the intelligence does not have, imagined during a dream:
/// what it would supply, and the form of what it would answer. It is only
/// ever simulated inside the dream; it cannot run in the real world.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct ImaginedCapabilitySpec {
    name: String,
    supplies: String,
    answers: Vec<Statement>,
    origin: String,
}

impl ImaginedCapabilitySpec {
    pub(crate) fn new(
        name: String,
        supplies: String,
        answers: Vec<Statement>,
        origin: String,
    ) -> Self {
        Self {
            name,
            supplies,
            answers,
            origin,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn supplies(&self) -> &str {
        &self.supplies
    }

    pub(crate) fn answers(&self) -> &[Statement] {
        &self.answers
    }
}
