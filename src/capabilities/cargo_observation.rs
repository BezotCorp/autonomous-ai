use std::{
    collections::BTreeSet,
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};

use crate::{
    acquisition::{
        cargo_lockfile::CargoLockfile, cargo_manifest::CargoManifest, github::Github,
        registry::Registry, repository::Repository,
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

/// Statements observed in one repository, or the failure.
type RepositoryObservation = Result<Vec<Statement>, String>;

/// Observes the Cargo manifests and lockfiles of repositories at their
/// current revision. Only what changed since the last acquisition is
/// fetched again: unchanged revisions, trees and files are recognised by
/// their SHA. Each repository is one observed scope: everything observed
/// at a revision describes the whole Cargo state of that revision.
///
/// Produced relations: `repository_revision(repo, sha)`,
/// `cargo_package(repo, manifest, package, version)`,
/// `declared_dependency(repo, manifest, package, requirement, kind)`,
/// `locked_package(repo, lockfile, package, version, source)`.
pub(crate) struct CargoObservation {
    descriptor: ModuleDescriptor,
}

impl CargoObservation {
    const MAXIMUM_CONCURRENT_ACQUISITIONS: usize = 4;
    const LOCAL_SOURCE: &'static str = "local";

    pub(crate) fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "cargo_observation",
                "Reads the Cargo manifests and lockfiles of GitHub repositories.",
            )
            .consuming(&[ContentKind::Need])
            .supplying(&[
                "repository_revision/2",
                "cargo_package/4",
                "declared_dependency/5",
                "locked_package/5",
            ])
            .producing(&[EpistemicKind::Observation, EpistemicKind::Need])
            .external(),
        }
    }

    /// Statements describing the Cargo files of one repository at a
    /// revision. Trees and files are addressed by their SHA: those already
    /// acquired are read from the store, the others fetched together.
    fn observe(
        github: &Github<'_>,
        repository: &Repository,
        revision: &str,
    ) -> RepositoryObservation {
        let files = github.file_blobs(repository, revision)?;
        let name = Term::symbol(repository.full_name());
        let mut statements = vec![Statement::new(
            "repository_revision",
            vec![name.clone(), Term::symbol(revision)],
        )];

        let relevant: Vec<(&String, &String)> = files
            .iter()
            .filter(|(path, _)| {
                CargoManifest::matches_path(path) || CargoLockfile::matches_path(path)
            })
            .collect();

        let shas: Vec<String> = relevant.iter().map(|(_, sha)| (*sha).clone()).collect();
        let contents = github.blobs(repository, &shas);

        for (path, sha) in relevant {
            let content = match contents.get(sha) {
                Some(Ok(content)) => content,
                Some(Err(error)) => {
                    return Err(format!("{}:{path}: {error}", repository.full_name()));
                }
                None => {
                    return Err(format!(
                        "{}:{path}: content not acquired",
                        repository.full_name()
                    ));
                }
            };

            if CargoManifest::matches_path(path) {
                let manifest = CargoManifest::parse(path.clone(), content)
                    .map_err(|error| format!("{path}: {error}"))?;
                let manifest_path = Term::text(manifest.path());

                if let Some(package) = manifest.package() {
                    let version = manifest
                        .version()
                        .map(|version| Term::Version(version.clone()))
                        .unwrap_or_else(|| Term::symbol("unversioned"));

                    statements.push(Statement::new(
                        "cargo_package",
                        vec![
                            name.clone(),
                            manifest_path.clone(),
                            Term::symbol(package),
                            version,
                        ],
                    ));
                }

                for dependency in manifest.dependencies() {
                    let requirement = dependency
                        .requirement()
                        .map(|requirement| Term::text(requirement.to_string()))
                        .unwrap_or_else(|| {
                            Term::symbol(if dependency.is_inherited() {
                                "inherited"
                            } else {
                                "unconstrained"
                            })
                        });

                    statements.push(Statement::new(
                        "declared_dependency",
                        vec![
                            name.clone(),
                            manifest_path.clone(),
                            Term::symbol(dependency.package()),
                            requirement,
                            Term::symbol(dependency.kind().name()),
                        ],
                    ));
                }
            } else {
                let lockfile = CargoLockfile::parse(path.clone(), content)
                    .map_err(|error| format!("{path}: {error}"))?;

                for package in lockfile.packages() {
                    let source = match package.source() {
                        None => Self::LOCAL_SOURCE.to_owned(),
                        Some(source) => Registry::from_source(source)
                            .map(|registry| registry.identifier().to_owned())
                            .unwrap_or_else(|| source.to_owned()),
                    };

                    statements.push(Statement::new(
                        "locked_package",
                        vec![
                            name.clone(),
                            Term::text(lockfile.path()),
                            Term::symbol(package.name()),
                            Term::Version(package.version().clone()),
                            Term::symbol(source),
                        ],
                    ));
                }
            }
        }

        Ok(statements)
    }

    /// Current revision of every repository, looked up together, then the
    /// observation of each one, a few at a time.
    fn observe_many(
        github: &Github<'_>,
        repositories: &[Repository],
    ) -> Vec<(Repository, Option<String>, RepositoryObservation)> {
        let revisions = github.head_revisions(repositories);
        let next = AtomicUsize::new(0);
        let workers = repositories
            .len()
            .min(Self::MAXIMUM_CONCURRENT_ACQUISITIONS);

        let mut results: Vec<_> = thread::scope(|scope| {
            let handles: Vec<_> = (0..workers)
                .map(|_| {
                    scope.spawn(|| {
                        let mut results = Vec::new();

                        while let Some(repository) =
                            repositories.get(next.fetch_add(1, Ordering::Relaxed))
                        {
                            let result = match revisions.get(repository) {
                                Some(Ok(revision)) => (
                                    Some(revision.clone()),
                                    Self::observe(github, repository, revision),
                                ),
                                Some(Err(error)) => (None, Err(error.clone())),
                                None => (None, Err("revision not looked up".to_owned())),
                            };

                            results.push((repository.clone(), result.0, result.1));
                        }

                        results
                    })
                })
                .collect();

            handles
                .into_iter()
                .flat_map(|handle| match handle.join() {
                    Ok(results) => results,
                    Err(payload) => std::panic::resume_unwind(payload),
                })
                .collect()
        });

        results.sort_by(|left, right| left.0.cmp(&right.0));
        results
    }
}

impl CognitiveModule for CargoObservation {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let graph = context.graph();
        let mut output = ModuleOutput::default();
        let mut targets = BTreeSet::new();
        let mut unfocused = Vec::new();

        for (need, pattern) in context.open_needs(&self.descriptor) {
            match pattern.arguments().first() {
                Some(Term::Symbol(repository)) => {
                    targets.insert(repository.clone());
                }
                _ => unfocused.push(need),
            }
        }

        if !unfocused.is_empty() {
            let known: Statement = "repository(?repository)".parse()?;
            let before = targets.len();

            for (_, bindings) in graph.matching(&known, &Default::default()) {
                if let Some(Term::Symbol(repository)) = bindings.get("repository") {
                    targets.insert(repository.clone());
                }
            }

            if targets.len() == before {
                output.propose(Proposal::new(
                    Content::Need(known),
                    EpistemicKind::Need,
                    unfocused,
                ));
            }
        }

        // Repositories already observed during this episode are current.
        let already: Statement = "repository_revision(?repository, ?revision)".parse()?;
        let observed: BTreeSet<String> = graph
            .matching(&already, &Default::default())
            .filter(|(id, _)| graph.node(*id).is_some_and(|node| node.is_current()))
            .filter_map(|(_, bindings)| match bindings.get("repository") {
                Some(Term::Symbol(repository)) => Some(repository.clone()),
                _ => None,
            })
            .collect();

        let repositories: Vec<Repository> = targets
            .iter()
            .filter(|repository| !observed.contains(*repository))
            .filter_map(|repository| {
                let (owner, name) = repository.split_once('/')?;
                Some(Repository::new(owner, name))
            })
            .collect();

        let mut failures = 0;
        let github = Github::new(context.access());

        for (repository, revision, result) in Self::observe_many(&github, &repositories) {
            match result {
                Ok(statements) => {
                    let provenance =
                        Provenance::new("cargo", Some(repository.full_name()), revision);

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
                Err(error) => {
                    failures += 1;
                    output.note(format!("{}: {error}", repository.full_name()));
                }
            }
        }

        if failures > 0 && failures == repositories.len() {
            return Err(format!(
                "every observation failed ({failures} repositories)"
            ));
        }

        Ok(output)
    }
}
