use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    memory::{
        evidence::Evidence, hypothesis_record::HypothesisRecord,
        hypothesis_status::HypothesisStatus, knowledge_entry::KnowledgeEntry, standing::Standing,
    },
    representation::{
        bindings::Bindings, provenance::Provenance, rule::Rule, statement::Statement,
    },
};

/// Declarative knowledge acquired across reasoning episodes. Initial
/// resources are kept outside of it: this memory only contains what the
/// intelligence observed, derived or conjectured by itself.
#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct KnowledgeMemory {
    facts: BTreeMap<Statement, KnowledgeEntry>,
    hypotheses: BTreeMap<Rule, HypothesisRecord>,
}

impl KnowledgeMemory {
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
