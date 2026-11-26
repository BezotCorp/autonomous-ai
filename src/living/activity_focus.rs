use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{events::event::Event, representation::statement::Statement};

/// What an activity is about.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ActivityFocus {
    /// An event and its questions.
    Event(Event),
    /// A concern.
    Concern { id: Uuid, subject: Statement },
    /// Nothing in particular.
    Free,
}

impl ActivityFocus {
    pub(crate) fn subject(&self) -> Option<Statement> {
        match self {
            Self::Event(event) => event.questions().first().cloned(),
            Self::Concern { subject, .. } => Some(subject.clone()),
            Self::Free => None,
        }
    }

    pub(crate) fn concern(&self) -> Option<Uuid> {
        match self {
            Self::Concern { id, .. } => Some(*id),
            _ => None,
        }
    }

    pub(crate) fn describe(&self) -> String {
        match self {
            Self::Event(event) => format!("event {}", event.id()),
            Self::Concern { subject, .. } => format!("concern {subject}"),
            Self::Free => "free".to_owned(),
        }
    }
}
