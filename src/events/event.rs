use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{events::event_payload::EventPayload, representation::statement::Statement};

/// Something that happened, received from the environment, optionally
/// accompanied by questions the intelligence is asked to answer.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Event {
    id: String,
    source: String,
    occurred_at: DateTime<Utc>,
    payload: EventPayload,
    #[serde(default)]
    questions: Vec<Statement>,
}

impl Event {
    /// Prefix of the source of events transmitted by another individual.
    pub(crate) const INDIVIDUAL_SOURCE: &'static str = "individual:";

    /// Event built from a transmission by another individual.
    pub(crate) fn transmitted(
        id: String,
        source: String,
        occurred_at: DateTime<Utc>,
        statements: Vec<Statement>,
        documents: Vec<crate::representation::document::Document>,
        questions: Vec<Statement>,
    ) -> Self {
        Self {
            id,
            source,
            occurred_at,
            payload: EventPayload::Assertions {
                statements,
                rules: Vec::new(),
                documents,
            },
            questions,
        }
    }

    /// Whether the event comes from outside any individual (a human or a
    /// system), as opposed to the individual itself or another one.
    pub(crate) fn is_external(&self) -> bool {
        self.source != "self" && !self.source.starts_with(Self::INDIVIDUAL_SOURCE)
    }

    /// Event the intelligence gives itself (e.g. when it decides to
    /// examine its own functioning).
    pub(crate) fn internal(id: String, questions: Vec<Statement>) -> Self {
        Self {
            id,
            source: "self".to_owned(),
            occurred_at: Utc::now(),
            payload: EventPayload::Assertions {
                statements: Vec::new(),
                rules: Vec::new(),
                documents: Vec::new(),
            },
            questions,
        }
    }

    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    pub(crate) fn source(&self) -> &str {
        &self.source
    }

    pub(crate) fn payload(&self) -> &EventPayload {
        &self.payload
    }

    pub(crate) fn questions(&self) -> &[Statement] {
        &self.questions
    }
}
