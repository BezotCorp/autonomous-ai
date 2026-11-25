use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct Repository {
    owner: String,
    name: String,
}

impl Repository {
    pub(crate) fn new(owner: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            name: name.into(),
        }
    }

    pub(crate) fn owner(&self) -> &str {
        &self.owner
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn full_name(&self) -> String {
        format!("{}/{}", self.owner(), self.name())
    }
}
