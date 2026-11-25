use serde::Serialize;

use crate::dependency_reference::DependencyReference;

#[derive(Clone, Default, Debug, Serialize)]
pub(crate) struct DependencyScan {
    references: Vec<DependencyReference>,
    failures: Vec<String>,
}

impl DependencyScan {
    pub(crate) fn references(&self) -> &[DependencyReference] {
        &self.references
    }

    pub(crate) fn failures(&self) -> &[String] {
        &self.failures
    }

    pub(crate) fn record(&mut self, reference: DependencyReference) {
        self.references.push(reference);
    }

    pub(crate) fn fail(&mut self, error: impl Into<String>) {
        self.failures.push(error.into());
    }
}
