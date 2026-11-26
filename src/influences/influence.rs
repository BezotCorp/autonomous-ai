use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::influences::influence_kind::InfluenceKind;

/// Element the unconscious injects into the context of the conscious, with
/// the situations it concerns (predicates), what it was drawn from (real
/// experiences, dreams, concerns) and why.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Influence {
    id: Uuid,
    kind: InfluenceKind,
    context: BTreeSet<String>,
    origins: Vec<String>,
    rationale: String,
    created_at: DateTime<Utc>,
    retired: Option<(DateTime<Utc>, String)>,
}

impl Influence {
    /// `context`: predicates of the situations where it applies, added to
    /// those the content itself is about.
    pub(crate) fn new(
        kind: InfluenceKind,
        context: BTreeSet<String>,
        origins: Vec<String>,
        rationale: String,
    ) -> Self {
        let mut context = context;

        context.extend(kind.predicates());

        Self {
            id: Uuid::now_v7(),
            kind,
            context,
            origins,
            rationale,
            created_at: Utc::now(),
            retired: None,
        }
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn kind(&self) -> &InfluenceKind {
        &self.kind
    }

    pub(crate) fn is_active(&self) -> bool {
        self.retired.is_none()
    }

    /// Whether the situation (its predicates) is one it concerns.
    pub(crate) fn applies_to(&self, situation: &BTreeSet<String>) -> bool {
        self.is_active() && !self.context.is_disjoint(situation)
    }

    pub(crate) fn retirement(&self) -> Option<&(DateTime<Utc>, String)> {
        self.retired.as_ref()
    }

    pub(crate) fn set_retirement(&mut self, retired: Option<(DateTime<Utc>, String)>) {
        self.retired = retired;
    }
}
