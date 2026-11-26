use serde::{Deserialize, Serialize};

/// Kind of content a cognitive module can consume.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ContentKind {
    Statement,
    Rule,
    Need,
    Document,
}
