use std::collections::BTreeSet;

use crate::{
    acquisition::{github::Github, repository::Repository, rust_surface::RustSurface},
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

/// Compares the sources of two releases of a repository, identified by
/// their version tags.
///
/// Produced relations: `release_revision(repo, version, sha)`,
/// `release_file_changed(repo, from, to, path)`,
/// `release_item_removed(repo, from, to, path, category, signature)`,
/// `release_item_added(repo, from, to, path, category, signature)`.
/// Interpreting what a change implies is left to the reasoning.
pub(crate) struct ReleaseObservation {
    descriptor: ModuleDescriptor,
}

impl ReleaseObservation {
    pub(crate) fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "release_observation",
                "Compares the Rust sources of two releases of a repository.",
            )
            .consuming(&[ContentKind::Need])
            .supplying(&[
                "release_file_changed/4",
                "release_item_removed/6",
                "release_item_added/6",
            ])
            .producing(&[EpistemicKind::Observation])
            .external(),
        }
    }

    fn compare(
        repository: &Repository,
        from: &semver::Version,
        to: &semver::Version,
    ) -> Result<(String, Vec<Statement>), String> {
        let text = |error: Box<dyn std::error::Error>| error.to_string();
        let previous_revision = Github::release_revision(repository, from).map_err(text)?;
        let current_revision = Github::release_revision(repository, to).map_err(text)?;
        let previous_files = Github::file_blobs(repository, &previous_revision).map_err(text)?;
        let current_files = Github::file_blobs(repository, &current_revision).map_err(text)?;

        let name = Term::symbol(repository.full_name());
        let range = [Term::Version(from.clone()), Term::Version(to.clone())];
        let mut statements = vec![
            Statement::new(
                "release_revision",
                vec![
                    name.clone(),
                    range[0].clone(),
                    Term::symbol(&previous_revision),
                ],
            ),
            Statement::new(
                "release_revision",
                vec![
                    name.clone(),
                    range[1].clone(),
                    Term::symbol(&current_revision),
                ],
            ),
        ];

        let paths: BTreeSet<&String> = previous_files.keys().chain(current_files.keys()).collect();

        for path in paths {
            if previous_files.get(path) == current_files.get(path) {
                continue;
            }

            let located = [
                vec![name.clone()],
                range.to_vec(),
                vec![Term::text(path.as_str())],
            ]
            .concat();

            statements.push(Statement::new("release_file_changed", located.clone()));

            if !path.ends_with(".rs") {
                continue;
            }

            let surface = |files: &std::collections::BTreeMap<String, String>, revision: &str| {
                if files.contains_key(path) {
                    Github::read_file(repository, revision, path)
                        .map_err(|error| error.to_string())
                        .and_then(|source| RustSurface::parse(&source))
                        .map_err(|error| format!("{path}@{revision}: {error}"))
                } else {
                    Ok(RustSurface::default())
                }
            };

            let before = surface(&previous_files, &previous_revision)?;
            let after = surface(&current_files, &current_revision)?;

            for (predicate, items) in [
                ("release_item_removed", before.missing_from(&after)),
                ("release_item_added", after.missing_from(&before)),
            ] {
                for (category, signature) in items {
                    statements.push(Statement::new(
                        predicate,
                        [
                            located.clone(),
                            vec![Term::symbol(category), Term::text(signature)],
                        ]
                        .concat(),
                    ));
                }
            }
        }

        Ok((
            format!("{previous_revision}..{current_revision}"),
            statements,
        ))
    }
}

impl CognitiveModule for ReleaseObservation {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let mut output = ModuleOutput::default();
        let mut comparisons = BTreeSet::new();

        for (_, pattern) in context.open_needs(&self.descriptor) {
            match pattern.arguments() {
                [
                    Term::Symbol(repository),
                    Term::Version(from),
                    Term::Version(to),
                    ..,
                ] => {
                    comparisons.insert((repository.clone(), from.clone(), to.clone()));
                }
                _ => output.note(format!(
                    "{pattern}: repository and both versions must be known"
                )),
            }
        }

        let mut failures = Vec::new();

        for (repository, from, to) in &comparisons {
            let Some((owner, name)) = repository.split_once('/') else {
                failures.push(format!("{repository}: not an owner/name repository"));
                continue;
            };

            match Self::compare(&Repository::new(owner, name), from, to) {
                Ok((revisions, statements)) => {
                    let provenance = Provenance::new(
                        "release",
                        Some(format!("{repository}@{from}..{to}")),
                        Some(revisions),
                    );

                    for statement in statements {
                        output.propose(
                            Proposal::new(
                                Content::Statement(statement),
                                EpistemicKind::Observation,
                                Vec::new(),
                            )
                            .with_provenance(provenance.clone()),
                        );
                    }
                }
                Err(error) => failures.push(format!("{repository} {from}..{to}: {error}")),
            }
        }

        if !failures.is_empty() && failures.len() == comparisons.len() {
            return Err(failures.join("; "));
        }

        for failure in failures {
            output.note(failure);
        }

        Ok(output)
    }
}
