use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{
    dependency_documentation::DependencyDocumentation, dependency_reference::DependencyReference,
    dependency_scanner::DependencyScanner, documentation_cache::DocumentationCache,
    documentation_query::DocumentationQuery, release_comparison::ReleaseComparison,
    repository::Repository, snapshot_identity::SnapshotIdentity,
};

#[derive(Debug, Serialize)]
pub(crate) struct DocumentationInvestigation {
    dependencies: Vec<DependencyDocumentation>,
    failures: Vec<String>,
    facts: BTreeSet<String>,
}

impl DocumentationInvestigation {
    pub(crate) fn relation_facts(&self) -> Vec<crate::relation_fact::RelationFact> {
        use crate::relation_fact::RelationFact;

        let mut facts = BTreeSet::new();

        for dependency in &self.dependencies {
            let documentation = dependency.documentation();

            let package = documentation.package().to_owned();
            let version = documentation.version().to_string();

            for reference in dependency.references() {
                for symbol in reference.identifiers() {
                    facts.insert(RelationFact::new(
                        "dependency_symbol_reference",
                        vec![
                            package.clone(),
                            version.clone(),
                            symbol.clone(),
                            reference.file().to_owned(),
                            reference.import_name().to_owned(),
                        ],
                    ));
                }
            }

            for rows in dependency.selection().rows().values() {
                for row in rows {
                    let (Some(symbol), Some(kind)) = (row.get("symbol"), row.get("kind")) else {
                        continue;
                    };

                    let description = row.get("documentation").cloned().unwrap_or_default();

                    facts.insert(RelationFact::new(
                        "documented_symbol",
                        vec![
                            package.clone(),
                            version.clone(),
                            symbol.clone(),
                            kind.clone(),
                            description,
                        ],
                    ));
                }
            }
        }

        facts.into_iter().collect()
    }

    pub(crate) fn acquire(
        cache: &mut DocumentationCache,
        scanner: &mut DependencyScanner,
        publisher: &Repository,
        comparison: Option<&ReleaseComparison>,
        consumer: &SnapshotIdentity,
        queries: &[DocumentationQuery],
    ) -> Self {
        let mut snapshots = BTreeSet::new();
        let mut failures = Vec::new();

        snapshots.insert((
            consumer.repository().clone(),
            consumer.revision().to_owned(),
        ));

        if let Some(comparison) = comparison {
            snapshots.insert((publisher.clone(), comparison.previous_revision().to_owned()));

            snapshots.insert((publisher.clone(), comparison.current_revision().to_owned()));
        } else {
            failures.push("Publisher revisions unavailable".to_owned());
        }

        let mut requests: BTreeMap<(String, semver::Version), Vec<DependencyReference>> =
            BTreeMap::new();

        for (repository, revision) in snapshots {
            let scan = scanner.acquire(&repository, &revision);

            failures.extend(
                scan.failures().iter().map(|failure| {
                    format!("{}@{}: {}", repository.full_name(), revision, failure,)
                }),
            );

            for reference in scan.references() {
                requests
                    .entry((reference.package().to_owned(), reference.version().clone()))
                    .or_default()
                    .push(reference.clone());
            }
        }

        let dependencies: Vec<_> = requests
            .into_iter()
            .map(|((package, version), references)| {
                let documentation = cache.acquire(&package, &version).clone();

                DependencyDocumentation::new(documentation, references, queries)
            })
            .collect();

        let mut facts = BTreeSet::new();

        if dependencies.is_empty() && failures.is_empty() {
            facts.insert("no_dependency_reference_observed".to_owned());
        }

        if dependencies
            .iter()
            .any(DependencyDocumentation::is_available)
        {
            facts.insert("dependency_documentation_available".to_owned());
        }

        if !failures.is_empty()
            || dependencies
                .iter()
                .any(DependencyDocumentation::is_incomplete)
        {
            facts.insert("dependency_documentation_incomplete".to_owned());
        }

        Self {
            dependencies,
            failures,
            facts,
        }
    }

    pub(crate) fn facts(&self) -> &BTreeSet<String> {
        &self.facts
    }
}
