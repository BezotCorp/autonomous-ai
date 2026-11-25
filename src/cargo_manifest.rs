use serde::{Deserialize, Serialize};
use std::error;

use crate::{
    constants::{PACKAGE_KEY, WORKSPACE_KEY},
    dependency::Dependency,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct CargoManifest {
    path: String,
    package: Option<String>,
    version: Option<semver::Version>,
    dependencies: Vec<Dependency>,
    document: String,
}

impl CargoManifest {
    pub(crate) fn parse(path: String, content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let document: toml::Value = toml::from_str(content)?;

        let package = document
            .get(PACKAGE_KEY)
            .and_then(|value| value.get("name"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned);

        let version = document
            .get(PACKAGE_KEY)
            .and_then(|value| value.get("version"))
            .and_then(toml::Value::as_str)
            .map(semver::Version::parse)
            .transpose()?;

        let mut dependencies = Vec::new();

        Self::collect_dependencies(&document, None, &mut dependencies)?;

        if let Some(targets) = document.get("target").and_then(toml::Value::as_table) {
            for (target, configuration) in targets {
                Self::collect_dependencies(configuration, Some(target), &mut dependencies)?;
            }
        }

        if let Some(workspace) = document.get(WORKSPACE_KEY) {
            Self::collect_dependencies(workspace, Some("workspace"), &mut dependencies)?;
        }

        Ok(Self {
            path,
            package,
            version,
            dependencies,
            document: content.to_owned(),
        })
    }

    fn collect_dependencies(
        document: &toml::Value,
        target: Option<&str>,
        dependencies: &mut Vec<Dependency>,
    ) -> Result<(), Box<dyn error::Error>> {
        for kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
            let Some(table) = document.get(kind).and_then(toml::Value::as_table) else {
                continue;
            };

            for (name, value) in table {
                dependencies.push(Dependency::from_manifest(name, value, kind, target)?);
            }
        }

        Ok(())
    }
}
