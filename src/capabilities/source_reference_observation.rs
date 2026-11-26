use std::collections::BTreeSet;

use crate::{
    acquisition::{
        github::Github, repository::Repository, source_reference_scanner::SourceReferenceScanner,
    },
    cognition::{
        cognitive_module::CognitiveModule, module_context::ModuleContext,
        module_descriptor::ModuleDescriptor, module_output::ModuleOutput, proposal::Proposal,
    },
    reasoning::epistemic_kind::EpistemicKind,
    representation::{
        content::Content, content_kind::ContentKind, provenance::Provenance, statement::Statement,
        term::Term,
    },
};

/// Finds which symbols of which dependency versions a repository's Rust
/// sources use.
///
/// Produced relation:
/// `symbol_reference(repo, file, package, version, registry, symbol)`.
pub(crate) struct SourceReferenceObservation {
    descriptor: ModuleDescriptor,
}

impl SourceReferenceObservation {
    pub(crate) fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "source_reference_observation",
                "Finds the dependency symbols referenced by the Rust sources of a repository.",
            )
            .consuming(&[ContentKind::Need])
            .supplying(&["symbol_reference/6"])
            .producing(&[EpistemicKind::Observation])
            .external(),
        }
    }
}

impl CognitiveModule for SourceReferenceObservation {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let mut output = ModuleOutput::default();
        let mut repositories = BTreeSet::new();

        for (_, pattern) in context.open_needs(&self.descriptor) {
            match pattern.arguments().first() {
                Some(Term::Symbol(repository)) => {
                    repositories.insert(repository.clone());
                }
                _ => output.note(format!("{pattern}: the repository must be known")),
            }
        }

        let mut failures = 0;
        let github = Github::new(context.access());

        for full_name in &repositories {
            let Some((owner, name)) = full_name.split_once('/') else {
                output.note(format!("{full_name}: not an owner/name repository"));
                failures += 1;
                continue;
            };

            let repository = Repository::new(owner, name);

            let revision = match github.revision(&repository, "HEAD") {
                Ok(revision) => revision,
                Err(error) => {
                    output.note(format!("{full_name}: {error}"));
                    failures += 1;
                    continue;
                }
            };

            let scan = SourceReferenceScanner::scan(&github, &repository, &revision);
            let provenance = Provenance::new("source", Some(full_name.clone()), Some(revision));

            for failure in scan.failures() {
                output.note(format!("{full_name}: {failure}"));
            }

            for reference in scan.references() {
                for symbol in reference.symbols() {
                    output.propose(
                        Proposal::new(
                            Content::Statement(Statement::new(
                                "symbol_reference",
                                vec![
                                    Term::symbol(full_name),
                                    Term::text(reference.file()),
                                    Term::symbol(reference.package()),
                                    Term::Version(reference.version().clone()),
                                    Term::symbol(reference.registry()),
                                    Term::symbol(symbol),
                                ],
                            )),
                            EpistemicKind::Observation,
                            Vec::new(),
                        )
                        .with_provenance(provenance.clone()),
                    );
                }
            }
        }

        if failures > 0 && failures == repositories.len() {
            return Err("no repository could be scanned".to_owned());
        }

        Ok(output)
    }
}
