use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};

use crate::{
    acquisition::{
        documentation_provider::DocumentationProvider, documentation_status::DocumentationStatus,
        package_documentation::PackageDocumentation,
    },
    cognition::{
        cognitive_module::CognitiveModule, module_context::ModuleContext,
        module_descriptor::ModuleDescriptor, module_output::ModuleOutput, proposal::Proposal,
    },
    reasoning::epistemic_kind::EpistemicKind,
    representation::{
        content::Content, content_kind::ContentKind, document::Document, provenance::Provenance,
        statement::Statement, term::Term,
    },
};

/// Reads the rustdoc documentation of an exact package version on docs.rs
/// with the queries of the initial resources.
///
/// Produced: `public_symbol(package, version, symbol)` — or its explicit
/// negation when complete documentation does not contain the symbol —,
/// `documented_symbol(package, version, symbol, kind)`, and the
/// documentation text itself as a document, linked by
/// `symbol_documentation(package, version, symbol, document)`.
pub(crate) struct DocumentationObservation {
    descriptor: ModuleDescriptor,
    cache: RefCell<BTreeMap<(String, semver::Version), PackageDocumentation>>,
}

impl DocumentationObservation {
    /// Documentation host, an external resource without known limit.
    const RESOURCE: &'static str = "docs.rs";

    pub(crate) fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "documentation_observation",
                "Reads the documentation of an exact package version on docs.rs.",
            )
            .consuming(&[ContentKind::Need])
            .supplying(&[
                "public_symbol/3",
                "documented_symbol/4",
                "symbol_documentation/4",
            ])
            .producing(&[EpistemicKind::Observation])
            .external(),
            cache: RefCell::new(BTreeMap::new()),
        }
    }
}

impl CognitiveModule for DocumentationObservation {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let queries = context.resources().documentation_queries();

        if queries.is_empty() {
            return Err("no documentation query available in the initial resources".to_owned());
        }

        let mut output = ModuleOutput::default();
        let mut requests: BTreeMap<(String, semver::Version), BTreeSet<String>> = BTreeMap::new();

        for (_, pattern) in context.open_needs(&self.descriptor) {
            match pattern.arguments() {
                [
                    Term::Symbol(package),
                    Term::Version(version),
                    Term::Symbol(symbol),
                    ..,
                ] => {
                    requests
                        .entry((package.clone(), version.clone()))
                        .or_default()
                        .insert(symbol.clone());
                }
                _ => output.note(format!(
                    "{pattern}: package, version and symbol must be known"
                )),
            }
        }

        let mut cache = self.cache.borrow_mut();
        let meter = context.access().meter();

        for ((package, version), symbols) in requests {
            let documentation = match cache.entry((package.clone(), version.clone())) {
                std::collections::btree_map::Entry::Occupied(entry) => {
                    meter.avoid();
                    entry.into_mut()
                }
                std::collections::btree_map::Entry::Vacant(_) if !context.access().is_live() => {
                    output.note(format!(
                        "{package}@{version}: documentation not recorded, left missing in replay"
                    ));
                    continue;
                }
                std::collections::btree_map::Entry::Vacant(entry) => {
                    let documentation = DocumentationProvider::acquire(&package, &version);

                    meter.record(Self::RESOURCE, documentation.source(), 1.0);
                    entry.insert(documentation)
                }
            };

            let provenance = Provenance::new(
                "docs.rs",
                Some(format!("{package}@{version}")),
                Some(documentation.source().to_owned()),
            );

            let complete = matches!(documentation.status(), DocumentationStatus::Available);

            if !complete {
                output.note(format!(
                    "{package}@{version}: documentation {:?}{}",
                    documentation.status(),
                    documentation
                        .error()
                        .map(|error| format!(" ({error})"))
                        .unwrap_or_default(),
                ));
            }

            let selection = documentation.select(&symbols, queries);
            let identity = [Term::symbol(&package), Term::Version(version.clone())];
            let mut found = BTreeSet::new();
            let mut observe = |content: Content| {
                output.propose(
                    Proposal::new(content, EpistemicKind::Observation, Vec::new())
                        .with_provenance(provenance.clone()),
                );
            };

            for rows in selection.rows().values() {
                for row in rows {
                    let Some(symbol) = row.get("symbol") else {
                        continue;
                    };

                    found.insert(symbol.clone());
                    let located = [identity.to_vec(), vec![Term::symbol(symbol)]].concat();

                    observe(Content::Statement(Statement::new(
                        "public_symbol",
                        located.clone(),
                    )));

                    if let Some(kind) = row.get("kind") {
                        observe(Content::Statement(Statement::new(
                            "documented_symbol",
                            [located.clone(), vec![Term::symbol(kind)]].concat(),
                        )));
                    }

                    if let Some(text) = row
                        .get("documentation")
                        .filter(|text| !text.is_empty() && *text != "Null")
                    {
                        let identifier = format!("docs.rs/{package}/{version}/{symbol}");

                        observe(Content::Document(Document::new(
                            identifier.clone(),
                            Some("en".to_owned()),
                            text.clone(),
                        )));
                        observe(Content::Statement(Statement::new(
                            "symbol_documentation",
                            [located, vec![Term::symbol(identifier)]].concat(),
                        )));
                    }
                }
            }

            if complete && selection.is_exhaustive() {
                for symbol in symbols.difference(&found) {
                    observe(Content::Statement(Statement::negative(
                        "public_symbol",
                        [identity.to_vec(), vec![Term::symbol(symbol)]].concat(),
                    )));
                }
            }

            for failure in selection.failures() {
                output.note(format!("{package}@{version}: {failure}"));
            }
        }

        Ok(output)
    }
}
