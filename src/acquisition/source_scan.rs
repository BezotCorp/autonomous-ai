use crate::acquisition::source_reference::SourceReference;

/// Result of scanning a repository revision: the references found and the
/// files or dependencies that could not be resolved.
#[derive(Debug, Default)]
pub(crate) struct SourceScan {
    references: Vec<SourceReference>,
    failures: Vec<String>,
}

impl SourceScan {
    pub(crate) fn references(&self) -> &[SourceReference] {
        &self.references
    }

    pub(crate) fn failures(&self) -> &[String] {
        &self.failures
    }

    pub(crate) fn record(&mut self, reference: SourceReference) {
        self.references.push(reference);
    }

    pub(crate) fn fail(&mut self, error: impl Into<String>) {
        self.failures.push(error.into());
    }
}
