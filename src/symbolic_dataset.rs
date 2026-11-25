use std::{collections::BTreeSet, error::Error, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    documentation_query::DocumentationQuery, evidence_weight::EvidenceWeight,
    lexical_rule::LexicalRule, relational_knowledge::RelationalKnowledge,
    relational_rule::RelationalRule, symbolic_rule::SymbolicRule,
};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct SymbolicDataset {
    rules: Vec<SymbolicRule>,
    documentation_queries: Vec<DocumentationQuery>,

    #[serde(default)]
    relational_rules: Vec<RelationalRule>,

    #[serde(default)]
    lexical_rules: Vec<LexicalRule>,

    #[serde(default)]
    evidence_weights: Vec<EvidenceWeight>,
}

impl SymbolicDataset {
    pub(crate) fn documentation_queries(&self) -> &[DocumentationQuery] {
        &self.documentation_queries
    }

    pub(crate) fn rules(&self) -> &[SymbolicRule] {
        &self.rules
    }

    pub(crate) fn relational_rules(&self) -> &[RelationalRule] {
        &self.relational_rules
    }

    pub(crate) fn lexical_rules(&self) -> &[LexicalRule] {
        &self.lexical_rules
    }

    pub(crate) fn evidence_weights(&self) -> &[EvidenceWeight] {
        &self.evidence_weights
    }

    pub(crate) fn generate(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
        let mut dataset: Self = ron::de::from_str(&fs::read_to_string(source)?)?;

        let extension_path = source.with_file_name("relational-knowledge.ron");

        let extension: RelationalKnowledge =
            ron::de::from_str(&fs::read_to_string(extension_path)?)?;

        let (relational_rules, lexical_rules, evidence_weights) = extension.into_parts();

        dataset.relational_rules = relational_rules;
        dataset.lexical_rules = lexical_rules;
        dataset.evidence_weights = evidence_weights;

        dataset.validate()?;

        let bytes = bincode_next::serde::encode_to_vec(&dataset, bincode_next::config::standard())?;

        if let Some(parent) = destination.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }

        fs::write(destination, bytes)?;

        Ok(())
    }

    pub(crate) fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let bytes = fs::read(path)?;

        let (dataset, consumed): (Self, usize) =
            bincode_next::serde::decode_from_slice(&bytes, bincode_next::config::standard())?;

        if consumed != bytes.len() {
            return Err("Unexpected trailing dataset data".into());
        }

        dataset.validate()?;

        Ok(dataset)
    }

    fn validate(&self) -> Result<(), Box<dyn Error>> {
        let mut query_names = BTreeSet::new();

        for query in &self.documentation_queries {
            if query.name().trim().is_empty() || query.expression().trim().is_empty() {
                return Err("Invalid documentation query".into());
            }

            if !query_names.insert(query.name()) {
                return Err(format!("Duplicate documentation query: {}", query.name(),).into());
            }
        }

        if self.documentation_queries.is_empty() {
            return Err("No documentation queries provided".into());
        }

        let mut ids = BTreeSet::new();

        if self.rules.is_empty() {
            return Err("The symbolic dataset contains no rules".into());
        }

        for rule in &self.rules {
            if rule.id().trim().is_empty()
                || rule.concludes().trim().is_empty()
                || rule.explanation().trim().is_empty()
                || rule.requires().is_empty()
                || rule.requires().iter().any(|value| value.trim().is_empty())
            {
                return Err(format!("Invalid symbolic rule: {}", rule.id(),).into());
            }

            if !ids.insert(rule.id()) {
                return Err(format!("Duplicate symbolic rule: {}", rule.id(),).into());
            }
        }

        let mut relational_ids = BTreeSet::new();

        for rule in &self.relational_rules {
            rule.validate()?;

            if !relational_ids.insert(rule.id()) {
                return Err(format!("Duplicate relational rule: {}", rule.id(),).into());
            }
        }

        let mut lexical_ids = BTreeSet::new();

        for rule in &self.lexical_rules {
            rule.validate()?;

            if !lexical_ids.insert(rule.id()) {
                return Err(format!("Duplicate lexical rule: {}", rule.id(),).into());
            }
        }

        for weight in &self.evidence_weights {
            weight.validate()?;
        }

        Ok(())
    }
}
