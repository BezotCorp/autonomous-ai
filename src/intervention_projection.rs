use std::collections::BTreeSet;

use serde::Serialize;

use crate::{
    documentation_investigation::DocumentationInvestigation, release_comparison::ReleaseComparison,
    repository::Repository, symbolic_deduction::SymbolicDeduction,
};

#[derive(Debug, Serialize)]
pub(crate) struct InterventionProjection {
    event_id: String,
    publisher: Repository,
    consumer: Repository,
    package: String,
    previous_version: semver::Version,
    current_version: semver::Version,
    lockfile: String,
    comparison: Option<ReleaseComparison>,
    documentation: Option<DocumentationInvestigation>,
    investigation_failure: Option<String>,
    facts: BTreeSet<String>,
    deductions: Vec<SymbolicDeduction>,
}

impl InterventionProjection {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        event_id: String,
        publisher: Repository,
        consumer: Repository,
        package: String,
        previous_version: semver::Version,
        current_version: semver::Version,
        lockfile: String,
        comparison: Option<ReleaseComparison>,
        documentation: Option<DocumentationInvestigation>,
        investigation_failure: Option<String>,
        facts: BTreeSet<String>,
        deductions: Vec<SymbolicDeduction>,
    ) -> Self {
        Self {
            event_id,
            publisher,
            consumer,
            package,
            previous_version,
            current_version,
            lockfile,
            comparison,
            documentation,
            investigation_failure,
            facts,
            deductions,
        }
    }
}
