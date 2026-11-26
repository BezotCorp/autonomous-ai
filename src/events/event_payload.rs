use serde::{Deserialize, Serialize};

use crate::{
    acquisition::repository::Repository,
    representation::{document::Document, rule::Rule, statement::Statement},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EventPayload {
    RepositoryChanged {
        repository: Repository,
        revision: String,
    },
    PackagePublished {
        repository: Repository,
        package: String,
        registry: String,
        version: semver::Version,
    },
    WorkflowCompleted {
        repository: Repository,
        run_id: u64,
        revision: String,
        conclusion: String,
    },
    /// Generic communication of statements, rules and documents.
    Assertions {
        #[serde(default)]
        statements: Vec<Statement>,
        #[serde(default)]
        rules: Vec<Rule>,
        #[serde(default)]
        documents: Vec<Document>,
    },
}
