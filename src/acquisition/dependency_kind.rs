use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DependencyKind {
    Normal,
    Development,
    Build,
}

impl DependencyKind {
    pub(crate) const ALL: [Self; 3] = [Self::Normal, Self::Development, Self::Build];

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Development => "development",
            Self::Build => "build",
        }
    }

    pub(crate) fn manifest_key(self) -> &'static str {
        match self {
            Self::Normal => "dependencies",
            Self::Development => "dev-dependencies",
            Self::Build => "build-dependencies",
        }
    }
}
