use serde::Serialize;

use crate::representation::{rule::Rule, statement::Statement};

/// How the persistent knowledge evolved at the end of an episode.
#[derive(Debug, Default, Serialize)]
pub(crate) struct ConsolidationSummary {
    new_observations: usize,
    new_derivations: usize,
    obsolete: Vec<Statement>,
    hypotheses_proposed: Vec<Rule>,
    hypotheses_corroborated: Vec<(Rule, Statement)>,
    hypotheses_refuted: Vec<Rule>,
}

impl ConsolidationSummary {
    pub(crate) fn observed(&mut self) {
        self.new_observations += 1;
    }

    pub(crate) fn derived(&mut self) {
        self.new_derivations += 1;
    }

    pub(crate) fn obsolete(&mut self, statements: Vec<Statement>) {
        self.obsolete.extend(statements);
    }

    pub(crate) fn proposed(&mut self, rule: Rule) {
        self.hypotheses_proposed.push(rule);
    }

    pub(crate) fn corroborated(&mut self, rule: Rule, statement: Statement) {
        self.hypotheses_corroborated.push((rule, statement));
    }

    pub(crate) fn refuted(&mut self, rule: Rule) {
        self.hypotheses_refuted.push(rule);
    }

    pub(crate) fn hypotheses_proposed(&self) -> &[Rule] {
        &self.hypotheses_proposed
    }

    pub(crate) fn hypotheses_refuted(&self) -> &[Rule] {
        &self.hypotheses_refuted
    }
}
