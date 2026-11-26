use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    memory::hypothesis_status::HypothesisStatus,
    representation::{rule::Rule, statement::Statement},
};

/// Persisted conjecture. A hypothesis never becomes a fact: it only
/// accumulates independent corroborations or gets refuted.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct HypothesisRecord {
    rule: Rule,
    proposed_in: Uuid,
    initial_support: u64,
    corroborations: BTreeSet<Statement>,
    refutations: Vec<String>,
    status: HypothesisStatus,
    proposed_at: DateTime<Utc>,
    refuted_at: Option<DateTime<Utc>>,
}

impl HypothesisRecord {
    pub(crate) fn new(rule: Rule, proposed_in: Uuid, initial_support: u64) -> Self {
        Self {
            rule,
            proposed_in,
            initial_support,
            corroborations: BTreeSet::new(),
            refutations: Vec::new(),
            status: HypothesisStatus::Open,
            proposed_at: Utc::now(),
            refuted_at: None,
        }
    }

    pub(crate) fn corroborations(&self) -> &BTreeSet<Statement> {
        &self.corroborations
    }

    /// Withdraws a refutation (when what refuted it turned out unfounded).
    pub(crate) fn reopen(&mut self) {
        self.status = HypothesisStatus::Open;
        self.refuted_at = None;
    }

    /// State of the hypothesis at an earlier instant, if it existed then.
    pub(crate) fn as_of(&self, at: DateTime<Utc>) -> Option<Self> {
        if self.proposed_at > at {
            return None;
        }

        let mut past = self.clone();

        if self.refuted_at.is_some_and(|refuted| refuted > at) {
            past.reopen();
        }

        Some(past)
    }

    pub(crate) fn rule(&self) -> &Rule {
        &self.rule
    }

    pub(crate) fn status(&self) -> &HypothesisStatus {
        &self.status
    }

    /// Number of distinct observations supporting the hypothesis.
    pub(crate) fn support(&self) -> u64 {
        self.initial_support + self.corroborations.len() as u64
    }

    /// Records an observed statement that the hypothesis had predicted.
    /// Repeating the same observation does not strengthen it again.
    pub(crate) fn corroborate(&mut self, statement: Statement) -> bool {
        self.corroborations.insert(statement)
    }

    pub(crate) fn refute(&mut self, reason: String) {
        self.refutations.push(reason);
        self.status = HypothesisStatus::Refuted;
        self.refuted_at = Some(Utc::now());
    }
}
