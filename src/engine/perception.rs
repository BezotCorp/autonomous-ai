use crate::{
    acquisition::{registry::Registry, repository::Repository},
    events::{event::Event, event_payload::EventPayload},
    mathematics::rational::Rational,
    representation::{content::Content, statement::Statement, term::Term},
};

/// Translates a received event into observed contents. Perception only
/// changes the representation; interpreting what the event implies is
/// left to the reasoning.
pub(crate) struct Perception;

impl Perception {
    pub(crate) fn perceive(event: &Event) -> Vec<Content> {
        let statements = match event.payload() {
            EventPayload::RepositoryChanged {
                repository,
                revision,
            } => vec![
                Self::owner(repository),
                Statement::new(
                    "repository_changed",
                    vec![Self::repository(repository), Term::symbol(revision)],
                ),
            ],

            EventPayload::PackagePublished {
                repository,
                package,
                registry,
                version,
            } => vec![
                Self::owner(repository),
                Statement::new(
                    "package_published",
                    vec![
                        Term::symbol(package),
                        Term::Version(version.clone()),
                        Term::symbol(Registry::from_index(registry).identifier()),
                    ],
                ),
                Statement::new(
                    "package_repository",
                    vec![Term::symbol(package), Self::repository(repository)],
                ),
            ],

            EventPayload::WorkflowCompleted {
                repository,
                run_id,
                revision,
                conclusion,
            } => vec![
                Self::owner(repository),
                Statement::new(
                    "workflow_completed",
                    vec![
                        Self::repository(repository),
                        Term::Number(Rational::integer(i128::from(*run_id))),
                        Term::symbol(revision),
                        Term::symbol(conclusion),
                    ],
                ),
            ],

            EventPayload::Assertions {
                statements,
                rules,
                documents,
            } => {
                return statements
                    .iter()
                    .cloned()
                    .map(Content::Statement)
                    .chain(rules.iter().cloned().map(Content::Rule))
                    .chain(documents.iter().cloned().map(Content::Document))
                    .collect();
            }
        };

        statements.into_iter().map(Content::Statement).collect()
    }

    fn repository(repository: &Repository) -> Term {
        Term::symbol(repository.full_name())
    }

    fn owner(repository: &Repository) -> Statement {
        Statement::new(
            "repository_owner",
            vec![
                Self::repository(repository),
                Term::symbol(repository.owner()),
            ],
        )
    }
}
