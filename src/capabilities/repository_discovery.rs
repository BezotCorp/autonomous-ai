use std::collections::BTreeSet;

use crate::{
    acquisition::github::Github,
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

/// Lists the repositories of an owner on GitHub. The owner comes from the
/// need itself or, failing that, from the owners already known.
pub(crate) struct RepositoryDiscovery {
    descriptor: ModuleDescriptor,
}

impl RepositoryDiscovery {
    pub(crate) fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "repository_discovery",
                "Lists the repositories belonging to an owner on GitHub.",
            )
            .consuming(&[ContentKind::Need])
            .supplying(&["repository/1", "repository_owner/2"])
            .producing(&[EpistemicKind::Observation])
            .external(),
        }
    }
}

impl CognitiveModule for RepositoryDiscovery {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let graph = context.graph();
        let mut output = ModuleOutput::default();
        let mut owners = BTreeSet::new();
        let mut unfocused = false;

        for (_, pattern) in context.open_needs(&self.descriptor) {
            match (pattern.predicate(), pattern.arguments()) {
                ("repository_owner", [_, Term::Symbol(owner)]) => {
                    owners.insert(owner.clone());
                }
                ("repository", [Term::Symbol(repository)]) => {
                    if let Some((owner, _)) = repository.split_once('/') {
                        owners.insert(owner.to_owned());
                    }
                }
                _ => unfocused = true,
            }
        }

        if unfocused {
            let known: Statement = "repository_owner(?repository, ?owner)"
                .parse()
                .map_err(|error: String| error)?;

            for (_, bindings) in graph.matching(&known, &Default::default()) {
                if let Some(Term::Symbol(owner)) = bindings.get("owner") {
                    owners.insert(owner.clone());
                }
            }
        }

        if owners.is_empty() {
            output.note("no owner known to list repositories from");
        }

        let mut failures = Vec::new();

        for owner in &owners {
            let listed = Statement::new("repositories_listed", vec![Term::symbol(owner)]);

            if graph.holding(&listed).next().is_some() {
                continue;
            }

            let repositories = match Github::repositories(owner) {
                Ok(repositories) => repositories,
                Err(error) => {
                    failures.push(format!("{owner}: {error}"));
                    continue;
                }
            };

            let provenance = Provenance::new("github", Some(format!("owner:{owner}")), None);
            let mut observe = |statement: Statement| {
                output.propose(
                    Proposal::new(
                        Content::Statement(statement),
                        EpistemicKind::Observation,
                        Vec::new(),
                    )
                    .with_provenance(provenance.clone()),
                );
            };

            for repository in &repositories {
                let name = Term::symbol(repository.full_name());

                observe(Statement::new("repository", vec![name.clone()]));
                observe(Statement::new(
                    "repository_owner",
                    vec![name, Term::symbol(owner)],
                ));
            }

            observe(listed);
        }

        if !failures.is_empty() && failures.len() == owners.len() {
            return Err(failures.join("; "));
        }

        for failure in failures {
            output.note(failure);
        }

        Ok(output)
    }
}
