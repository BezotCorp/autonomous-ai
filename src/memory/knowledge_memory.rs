use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    memory::{
        change_prior::ChangePrior, evidence::Evidence, hypothesis_record::HypothesisRecord,
        hypothesis_status::HypothesisStatus, knowledge_entry::KnowledgeEntry,
        scope_history::ScopeHistory, standing::Standing,
    },
    representation::{
        bindings::Bindings, provenance::Provenance, rule::Rule, statement::Statement,
    },
};

/// Declarative knowledge acquired across reasoning episodes. Initial
/// resources are kept outside of it: this memory only contains what the
/// intelligence observed, derived or conjectured by itself.
///
/// What was observed is dated: the memory keeps, for every observed scope
/// of a source, when it was last verified and how often it changed, so
/// that a recollection is never mistaken for a verification of the
/// present state.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct KnowledgeMemory {
    facts: BTreeMap<Statement, KnowledgeEntry>,
    hypotheses: BTreeMap<Rule, HypothesisRecord>,
    scopes: BTreeMap<(String, String), ScopeHistory>,
}

impl KnowledgeMemory {
    /// Depth up to which the staleness of derived knowledge is traced back
    /// to the observations it rests on.
    const MAXIMUM_DERIVATION_DEPTH: usize = 6;

    /// Records that a scope of a source was observed at a revision.
    /// Returns whether the revision changed since the previous observation.
    pub(crate) fn verify_scope(
        &mut self,
        source: &str,
        scope: &str,
        revision: Option<&str>,
        at: DateTime<Utc>,
    ) -> bool {
        match self.scopes.get_mut(&(source.to_owned(), scope.to_owned())) {
            Some(history) => history.verify(revision, at),
            None => {
                self.scopes.insert(
                    (source.to_owned(), scope.to_owned()),
                    ScopeHistory::new(revision.map(str::to_owned), at),
                );
                false
            }
        }
    }

    pub(crate) fn scopes(&self) -> impl Iterator<Item = (&(String, String), &ScopeHistory)> {
        self.scopes.iter()
    }

    /// Provenance under which an entry would be recalled now, dated by the
    /// last verification of its scope, and the probability that it no
    /// longer holds. Content without an observed scope (events, initial
    /// resources) records something that happened and does not expire;
    /// derived content is as outdated as the observations it rests on.
    pub(crate) fn currency(
        &self,
        entry: &KnowledgeEntry,
        now: DateTime<Utc>,
        prior: ChangePrior,
    ) -> (Option<Provenance>, f64) {
        self.currency_at_depth(entry, now, prior, 0)
    }

    fn currency_at_depth(
        &self,
        entry: &KnowledgeEntry,
        now: DateTime<Utc>,
        prior: ChangePrior,
        depth: usize,
    ) -> (Option<Provenance>, f64) {
        let mut best: Option<(Option<Provenance>, f64)> = None;

        for evidence in entry.evidence() {
            let candidate = match evidence.provenance() {
                Some(provenance) => match provenance.scope() {
                    None => (Some(provenance.clone()), 0.0),
                    Some(scope) => {
                        match self
                            .scopes
                            .get(&(provenance.source().to_owned(), scope.to_owned()))
                        {
                            Some(history)
                                if provenance.revision().is_none()
                                    || provenance.revision() == history.revision() =>
                            {
                                (
                                    Some(
                                        provenance.clone().confirmed_at(history.last_verified_at()),
                                    ),
                                    history.staleness(now, prior.changes(), prior.seconds()),
                                )
                            }
                            // Observed in a revision the scope has left since.
                            Some(_) => (Some(provenance.clone()), 1.0),
                            None => (
                                Some(provenance.clone()),
                                1.0 - (-prior.changes() / prior.seconds().max(1e-9)
                                    * (now - provenance.acquired_at()).num_seconds().max(0) as f64)
                                    .exp(),
                            ),
                        }
                    }
                },
                None if depth >= Self::MAXIMUM_DERIVATION_DEPTH => (None, 1.0),
                None => {
                    let staleness = 1.0
                        - evidence
                            .premises()
                            .iter()
                            .map(|premise| match self.facts.get(premise) {
                                Some(premise) if premise.standing().is_active() => {
                                    1.0 - self.currency_at_depth(premise, now, prior, depth + 1).1
                                }
                                Some(_) => 0.0,
                                // Not acquired: an initial resource.
                                None => 1.0,
                            })
                            .product::<f64>();

                    (None, staleness)
                }
            };

            if best
                .as_ref()
                .is_none_or(|(_, staleness)| candidate.1 < *staleness)
            {
                best = Some(candidate);
            }
        }

        best.unwrap_or((None, 0.0))
    }
    /// The knowledge as it stood at an earlier instant: only evidence
    /// recorded by then, standings and refutations dated before it.
    pub(crate) fn as_of(&self, instant: DateTime<Utc>) -> Self {
        let mut past = Self::default();

        for (statement, entry) in &self.facts {
            if let Some(entry) = entry.as_of(instant) {
                past.facts.insert(statement.clone(), entry);
            }
        }

        for (rule, record) in &self.hypotheses {
            if let Some(record) = record.as_of(instant) {
                past.hypotheses.insert(rule.clone(), record);
            }
        }

        for (scope, history) in &self.scopes {
            if let Some(history) = history.as_of(instant) {
                past.scopes.insert(scope.clone(), history);
            }
        }

        past
    }

    /// Confronts a rule with what is believed: the premise matches whose
    /// conclusion holds (confirmations, each with the facts involved) and the
    /// premise matches whose conclusion is known to be false
    /// (counterexamples, each with the facts showing it). The exploration
    /// is bounded.
    pub(crate) fn confront(
        &self,
        rule: &Rule,
        limit: usize,
    ) -> (Vec<Vec<Statement>>, Vec<Vec<Statement>>) {
        let mut partial: Vec<(Bindings, Vec<Statement>)> = vec![(Bindings::default(), Vec::new())];
        let mut explored = 0;

        for premise in rule.premises() {
            let mut next = Vec::new();

            for (bindings, facts) in &partial {
                let pattern = bindings.apply(premise);

                for entry in self.recall(&pattern) {
                    explored += 1;

                    if explored > limit {
                        return (Vec::new(), Vec::new());
                    }

                    if let Some(extended) = bindings.unify(premise, entry.statement()) {
                        let mut facts = facts.clone();

                        facts.push(entry.statement().clone());
                        next.push((extended, facts));
                    }
                }
            }

            partial = next;
        }

        let mut confirmations = Vec::new();
        let mut counterexamples = Vec::new();

        for (bindings, mut facts) in partial {
            let conclusion = bindings.apply(rule.conclusion());

            if !conclusion.is_ground() {
                continue;
            }

            let held = |statement: &Statement| {
                self.facts
                    .get(statement)
                    .is_some_and(|entry| entry.standing().is_active())
            };

            if held(&conclusion) {
                facts.push(conclusion);
                confirmations.push(facts);
            } else if held(&conclusion.complement()) {
                facts.push(conclusion.complement());
                counterexamples.push(facts);
            }
        }

        (confirmations, counterexamples)
    }

    pub(crate) fn entry(&self, statement: &Statement) -> Option<&KnowledgeEntry> {
        self.facts.get(statement)
    }

    /// Puts an entry back as it was (or removes it), to revert a correction.
    pub(crate) fn restore_entry(&mut self, statement: &Statement, entry: Option<KnowledgeEntry>) {
        match entry {
            Some(entry) => {
                self.facts.insert(statement.clone(), entry);
            }
            None => {
                self.facts.remove(statement);
            }
        }
    }

    /// Stops holding a belief, keeping its evidence and the reason.
    pub(crate) fn withdraw(&mut self, statement: &Statement, reason: String) -> bool {
        match self.facts.get_mut(statement) {
            Some(entry) if entry.standing().is_active() => {
                entry.set_standing(Standing::Obsolete {
                    since: Utc::now(),
                    reason,
                });
                true
            }
            _ => false,
        }
    }

    pub(crate) fn hypothesis(&self, rule: &Rule) -> Option<&HypothesisRecord> {
        self.hypotheses.get(rule)
    }

    pub(crate) fn hypothesis_mut(&mut self, rule: &Rule) -> Option<&mut HypothesisRecord> {
        self.hypotheses.get_mut(rule)
    }

    pub(crate) fn restore_hypothesis(&mut self, rule: &Rule, record: Option<HypothesisRecord>) {
        match record {
            Some(record) => {
                self.hypotheses.insert(rule.clone(), record);
            }
            None => {
                self.hypotheses.remove(rule);
            }
        }
    }

    pub(crate) fn entries(&self) -> impl Iterator<Item = &KnowledgeEntry> {
        self.facts.values()
    }

    pub(crate) fn active_entries(&self) -> impl Iterator<Item = &KnowledgeEntry> {
        self.facts
            .values()
            .filter(|entry| entry.standing().is_active())
    }

    pub(crate) fn hypotheses(&self) -> impl Iterator<Item = &HypothesisRecord> {
        self.hypotheses.values()
    }

    /// Active statements unifying with a pattern.
    pub(crate) fn recall<'a>(
        &'a self,
        pattern: &'a Statement,
    ) -> impl Iterator<Item = &'a KnowledgeEntry> + 'a {
        let bindings = Bindings::default();

        self.active_entries()
            .filter(move |entry| bindings.unify(pattern, entry.statement()).is_some())
    }

    /// Integrates an observation. Returns whether it brought new evidence.
    pub(crate) fn record_observation(
        &mut self,
        statement: Statement,
        provenance: Option<Provenance>,
        episode: Uuid,
    ) -> bool {
        if let Some(contrary) = self.facts.get_mut(&statement.complement())
            && contrary.standing().is_active()
        {
            contrary.set_standing(Standing::Obsolete {
                since: Utc::now(),
                reason: format!("contradicted by the later observation of {statement}"),
            });
        }

        let entry = self
            .facts
            .entry(statement.clone())
            .or_insert_with(|| KnowledgeEntry::new(statement, Standing::Observed));

        entry.set_standing(Standing::Observed);
        entry.support(Evidence::new(episode, provenance, Vec::new()))
    }

    /// Integrates a conclusion established from active knowledge only.
    pub(crate) fn record_derivation(
        &mut self,
        statement: Statement,
        premises: Vec<Statement>,
        episode: Uuid,
    ) -> bool {
        let entry = self
            .facts
            .entry(statement.clone())
            .or_insert_with(|| KnowledgeEntry::new(statement, Standing::Derived));

        if !entry.standing().is_active() {
            entry.set_standing(Standing::Derived);
        }

        entry.support(Evidence::new(episode, None, premises))
    }

    /// Marks as obsolete every statement observed in an earlier revision
    /// of a scope that has just been observed again without it, then
    /// propagates to derived statements that relied on it.
    pub(crate) fn supersede_scope(
        &mut self,
        source: &str,
        scope: &str,
        revision: &str,
        observed: &BTreeSet<Statement>,
    ) -> Vec<Statement> {
        let mut obsolete = Vec::new();

        for entry in self.facts.values_mut() {
            if !entry.standing().is_active() || observed.contains(entry.statement()) {
                continue;
            }

            let from_scope = entry.evidence().iter().all(|evidence| {
                evidence.provenance().is_some_and(|provenance| {
                    provenance.source() == source
                        && provenance.scope() == Some(scope)
                        && provenance.revision() != Some(revision)
                })
            });

            if from_scope && !entry.evidence().is_empty() {
                entry.set_standing(Standing::Obsolete {
                    since: Utc::now(),
                    reason: format!("{source} no longer observes it in {scope}@{revision}"),
                });
                obsolete.push(entry.statement().clone());
            }
        }

        let mut pending = obsolete.clone();

        while let Some(statement) = pending.pop() {
            for entry in self.facts.values_mut() {
                if !matches!(entry.standing(), Standing::Derived) {
                    continue;
                }

                let depends = entry
                    .evidence()
                    .iter()
                    .all(|evidence| evidence.premises().contains(&statement));

                if depends {
                    entry.set_standing(Standing::Obsolete {
                        since: Utc::now(),
                        reason: format!("premise {statement} is obsolete"),
                    });
                    obsolete.push(entry.statement().clone());
                    pending.push(entry.statement().clone());
                }
            }
        }

        obsolete
    }

    /// Registers a conjecture unless it is already known. Returns whether it was new.
    pub(crate) fn propose_hypothesis(&mut self, rule: Rule, episode: Uuid, support: u64) -> bool {
        if self.hypotheses.contains_key(&rule) {
            return false;
        }

        self.hypotheses
            .insert(rule.clone(), HypothesisRecord::new(rule, episode, support));
        true
    }

    pub(crate) fn corroborate(&mut self, rule: &Rule, statement: Statement) -> bool {
        self.hypotheses
            .get_mut(rule)
            .is_some_and(|record| record.corroborate(statement))
    }

    pub(crate) fn refute(&mut self, rule: &Rule, reason: String) {
        if let Some(record) = self.hypotheses.get_mut(rule) {
            record.refute(reason);
        }
    }

    pub(crate) fn is_refuted(&self, rule: &Rule) -> bool {
        self.hypotheses
            .get(rule)
            .is_some_and(|record| record.status() == &HypothesisStatus::Refuted)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use uuid::Uuid;

    use super::KnowledgeMemory;
    use crate::representation::provenance::Provenance;

    #[test]
    fn supersedes_facts_missing_from_a_new_revision() {
        let mut memory = KnowledgeMemory::default();
        let episode = Uuid::now_v7();
        let old = Provenance::new("cargo", Some("o/r".to_owned()), Some("a".to_owned()));
        let kept = "uses(o/r, serde)".parse().unwrap();
        let dropped = "uses(o/r, toml)".parse().unwrap();

        memory.record_observation(kept, Some(old.clone()), episode);
        memory.record_observation(dropped, Some(old), episode);
        memory.record_derivation(
            "outdated(o/r)".parse().unwrap(),
            vec!["uses(o/r, toml)".parse().unwrap()],
            episode,
        );

        let observed = BTreeSet::from(["uses(o/r, serde)".parse().unwrap()]);
        let obsolete = memory.supersede_scope("cargo", "o/r", "b", &observed);

        assert_eq!(obsolete.len(), 2);
        assert_eq!(memory.active_entries().count(), 1);
    }
}
