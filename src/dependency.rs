use serde::{Deserialize, Serialize};

use crate::constants::{PACKAGE_KEY, WORKSPACE_KEY};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Dependency {
    name: String,
    package: String,
    requirement: Option<semver::VersionReq>,
    source: Option<String>,
    kind: String,
    target: Option<String>,
    workspace: bool,
    optional: bool,
    features: Vec<String>,
}

impl Dependency {
    pub(crate) fn from_manifest(
        name: &str,
        value: &toml::Value,
        kind: &str,
        target: Option<&str>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let table = value.as_table();

        let field = |key: &str| table.and_then(|table| table.get(key));

        let package = field(PACKAGE_KEY)
            .and_then(toml::Value::as_str)
            .unwrap_or(name)
            .to_owned();

        let requirement = value
            .as_str()
            .or_else(|| field("version").and_then(toml::Value::as_str))
            .map(semver::VersionReq::parse)
            .transpose()?;

        let source = ["git", "path", "registry"].into_iter().find_map(|key| {
            field(key)
                .and_then(toml::Value::as_str)
                .map(|value| format!("{key}:{value}"))
        });

        let features = field("features")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(toml::Value::as_str)
            .map(str::to_owned)
            .collect();

        Ok(Self {
            name: name.to_owned(),
            package,
            requirement,
            source,
            kind: kind.to_owned(),
            target: target.map(str::to_owned),
            workspace: field(WORKSPACE_KEY)
                .and_then(toml::Value::as_bool)
                .unwrap_or(false),
            optional: field("optional")
                .and_then(toml::Value::as_bool)
                .unwrap_or(false),
            features,
        })
    }
}
