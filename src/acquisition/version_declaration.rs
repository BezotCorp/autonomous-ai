use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::acquisition::constants::WORKSPACE_KEY;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum VersionDeclaration {
    Explicit(semver::Version),
    Inherited,
}

impl VersionDeclaration {
    pub(crate) fn parse(value: &toml::Value) -> Result<Self, Box<dyn Error>> {
        if let Some(version) = value.as_str() {
            return Ok(Self::Explicit(semver::Version::parse(version)?));
        }

        if value.get(WORKSPACE_KEY).and_then(toml::Value::as_bool) == Some(true) {
            return Ok(Self::Inherited);
        }

        Err("Invalid package version declaration".into())
    }
}
