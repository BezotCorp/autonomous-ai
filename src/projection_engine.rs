use std::collections::{BTreeMap, BTreeSet};

use crate::{
    dependency_scanner::DependencyScanner, documentation_cache::DocumentationCache,
    documentation_investigation::DocumentationInvestigation, event_payload::EventPayload,
    intervention_projection::InterventionProjection, memory::Memory,
    release_comparison::ReleaseComparison, symbolic_dataset::SymbolicDataset,
    symbolic_engine::SymbolicEngine,
};

pub(crate) struct ProjectionEngine;

impl ProjectionEngine {
    pub(crate) fn project(
        memory: &Memory,
        dataset: &SymbolicDataset,
    ) -> Vec<InterventionProjection> {
        let mut projections = Vec::new();
        let mut documentation_cache = DocumentationCache::default();
        let mut dependency_scanner = DependencyScanner::default();

        for analysis in memory.publication_analyses() {
            let Some(event) = memory.event(analysis.event_id()) else {
                continue;
            };

            let EventPayload::PackagePublished {
                repository,
                package,
                version,
                ..
            } = event.payload()
            else {
                continue;
            };

            let mut comparisons = BTreeMap::new();

            for consumer in analysis.older_packages() {
                comparisons
                    .entry(consumer.version().clone())
                    .or_insert_with(|| {
                        ReleaseComparison::acquire(repository, consumer.version(), version)
                            .map_err(|error| error.to_string())
                    });
            }

            for consumer in analysis.older_packages() {
                let mut facts = BTreeSet::new();

                facts.insert("older_resolved_dependency".to_owned());
                facts.insert("consumer_usage_unverified".to_owned());

                let (comparison, failure) = match comparisons.get(consumer.version()) {
                    Some(Ok(comparison)) => {
                        facts.extend(comparison.facts().iter().cloned());

                        (Some(comparison.clone()), None)
                    }

                    Some(Err(error)) => {
                        facts.insert("release_comparison_unavailable".to_owned());

                        (None, Some(error.clone()))
                    }

                    None => {
                        facts.insert("release_comparison_unavailable".to_owned());

                        (None, Some("Missing comparison".to_owned()))
                    }
                };

                let initial_deductions = SymbolicEngine::infer(dataset, &facts);

                let documentation = if initial_deductions
                    .iter()
                    .any(|deduction| deduction.conclusion() == "observe: dependency_documentation")
                {
                    let investigation = DocumentationInvestigation::acquire(
                        &mut documentation_cache,
                        &mut dependency_scanner,
                        repository,
                        comparison.as_ref(),
                        consumer.snapshot(),
                        dataset.documentation_queries(),
                    );

                    facts.extend(investigation.facts().iter().cloned());

                    Some(investigation)
                } else {
                    None
                };

                let deductions = SymbolicEngine::infer(dataset, &facts);

                projections.push(InterventionProjection::new(
                    analysis.event_id().to_owned(),
                    repository.clone(),
                    consumer.snapshot().repository().clone(),
                    package.clone(),
                    consumer.version().clone(),
                    version.clone(),
                    consumer.lockfile().to_owned(),
                    comparison,
                    documentation,
                    failure,
                    facts,
                    deductions,
                ));
            }
        }

        projections
    }
}
