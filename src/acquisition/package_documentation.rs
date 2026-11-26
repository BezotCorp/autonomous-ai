use std::{collections::BTreeSet, sync::Arc};

use serde::Serialize;
use serde_json::Value;

use crate::acquisition::{
    documentation_query::DocumentationQuery, documentation_selection::DocumentationSelection,
    documentation_status::DocumentationStatus, rustdoc_graph::RustdocGraph,
};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PackageDocumentation {
    package: String,
    version: semver::Version,
    source: String,
    status: DocumentationStatus,
    format_version: Option<u64>,
    error: Option<String>,
    #[serde(skip)]
    graph: Option<Arc<RustdocGraph>>,
}

impl PackageDocumentation {
    pub(crate) fn new(package: &str, version: &semver::Version, source: String) -> Self {
        Self {
            package: package.to_owned(),
            version: version.clone(),
            source,
            status: DocumentationStatus::Unavailable,
            format_version: None,
            error: None,
            graph: None,
        }
    }

    pub(crate) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub(crate) fn source(&self) -> &str {
        &self.source
    }

    pub(crate) fn status(&self) -> DocumentationStatus {
        self.status
    }

    pub(crate) fn fail(&mut self, status: DocumentationStatus, error: impl Into<String>) {
        self.status = status;
        self.error = Some(error.into());
        self.graph = None;
    }

    pub(crate) fn integrate(&mut self, document: Value) {
        self.format_version = document.get("format_version").and_then(Value::as_u64);

        match RustdocGraph::new(document) {
            Ok(graph) => {
                self.status = if graph.is_empty() {
                    DocumentationStatus::Partial
                } else {
                    DocumentationStatus::Available
                };

                self.graph = Some(Arc::new(graph));
            }

            Err(error) => {
                self.fail(DocumentationStatus::Unsupported, error);
            }
        }
    }

    pub(crate) fn select(
        &self,
        identifiers: &BTreeSet<String>,
        queries: &[DocumentationQuery],
    ) -> DocumentationSelection {
        self.graph
            .as_ref()
            .map(|graph| graph.select(identifiers, queries))
            .unwrap_or_default()
    }
}
