use std::collections::BTreeSet;

use serde::Serialize;

use crate::{
    dependency_reference::DependencyReference, documentation_query::DocumentationQuery,
    documentation_selection::DocumentationSelection, documentation_status::DocumentationStatus,
    package_documentation::PackageDocumentation,
};

#[derive(Debug, Serialize)]
pub(crate) struct DependencyDocumentation {
    documentation: PackageDocumentation,
    references: Vec<DependencyReference>,
    selection: DocumentationSelection,
}

impl DependencyDocumentation {
    pub(crate) fn new(
        documentation: PackageDocumentation,
        references: Vec<DependencyReference>,
        queries: &[DocumentationQuery],
    ) -> Self {
        let identifiers: BTreeSet<String> = references
            .iter()
            .flat_map(|reference| reference.identifiers().iter().cloned())
            .collect();

        let selection = documentation.select(&identifiers, queries);

        Self {
            documentation,
            references,
            selection,
        }
    }

    pub(crate) fn is_available(&self) -> bool {
        matches!(
            self.documentation.status(),
            DocumentationStatus::Available | DocumentationStatus::Partial
        )
    }

    pub(crate) fn is_incomplete(&self) -> bool {
        !matches!(self.documentation.status(), DocumentationStatus::Available)
            || self.selection.is_incomplete()
    }
}
