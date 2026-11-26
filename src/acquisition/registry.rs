use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct Registry(String);

impl Registry {
    const CRATES_IO_GIT_INDEX: &str = "https://github.com/rust-lang/crates.io-index";

    const CRATES_IO_SPARSE_INDEX: &str = "https://index.crates.io";

    pub(crate) fn from_index(index: &str) -> Self {
        let index = index
            .strip_prefix("registry+")
            .or_else(|| index.strip_prefix("sparse+"))
            .unwrap_or(index)
            .trim_end_matches('/');

        let normalized = match index {
            "crates.io" | Self::CRATES_IO_SPARSE_INDEX => Self::CRATES_IO_GIT_INDEX,
            _ => index,
        };

        Self(normalized.to_owned())
    }

    pub(crate) fn identifier(&self) -> &str {
        &self.0
    }

    pub(crate) fn from_source(source: &str) -> Option<Self> {
        if source.starts_with("registry+") || source.starts_with("sparse+") {
            Some(Self::from_index(source))
        } else {
            None
        }
    }
}
