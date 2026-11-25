use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use serde_json::Value;
use trustfall::{FieldValue, execute_query};
use trustfall_rustdoc_adapter::{Crate, PackageIndex, PackageStorage, RustdocAdapter};

use crate::{
    documentation_query::DocumentationQuery, documentation_selection::DocumentationSelection,
};

pub(crate) struct RustdocGraph {
    document: Crate,
}

impl std::fmt::Debug for RustdocGraph {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RustdocGraph")
            .field("format_version", &self.document.format_version)
            .field("items", &self.document.index.len())
            .finish()
    }
}

impl RustdocGraph {
    const MAX_ROWS_PER_QUERY: usize = 256;

    pub(crate) fn new(document: Value) -> Result<Self, String> {
        let format = document
            .get("format_version")
            .and_then(Value::as_u64)
            .ok_or("Missing rustdoc format version")?;

        if format != 61 {
            return Err(format!("Unsupported rustdoc format: {format}",));
        }

        let document: Crate =
            serde_json::from_value(document).map_err(|error| error.to_string())?;

        Ok(Self { document })
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.document.index.is_empty()
    }

    pub(crate) fn select(
        &self,
        identifiers: &BTreeSet<String>,
        queries: &[DocumentationQuery],
    ) -> DocumentationSelection {
        let mut selection = DocumentationSelection::default();

        if identifiers.is_empty() {
            return selection;
        }

        let storage = PackageStorage::from_rustdoc(self.document.clone());

        let index = PackageIndex::from_storage(&storage);

        let adapter = RustdocAdapter::new(&index, None);
        let adapter = Arc::new(&adapter);

        let schema = RustdocAdapter::schema();

        let mut matched = BTreeSet::new();

        let identifiers_value = FieldValue::from(identifiers.iter().cloned().collect::<Vec<_>>());

        for query in queries {
            let arguments = BTreeMap::from([("symbols", identifiers_value.clone())]);

            let result = execute_query(schema, adapter.clone(), query.expression(), arguments);

            let iterator = match result {
                Ok(iterator) => iterator,

                Err(error) => {
                    selection.fail(format!("{}: {}", query.name(), error,));

                    continue;
                }
            };

            let mut rows = Vec::new();

            for (position, row) in iterator.enumerate() {
                if position >= Self::MAX_ROWS_PER_QUERY {
                    selection.truncate();
                    break;
                }

                if let Some(symbol) = row.get("symbol").and_then(FieldValue::as_str) {
                    matched.insert(symbol.to_owned());
                }

                let row = row
                    .into_iter()
                    .map(|(key, value)| {
                        let value = match value.as_str() {
                            Some(value) => value.to_owned(),
                            None => format!("{value:?}"),
                        };

                        (key.to_string(), value)
                    })
                    .collect();

                rows.push(row);
            }

            selection.insert(query.name().to_owned(), rows);
        }

        for identifier in identifiers.difference(&matched) {
            selection.unresolved(identifier.clone());
        }

        selection
    }
}
