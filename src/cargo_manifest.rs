use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::{
    constants::{PACKAGE_KEY, VERSION_KEY, WORKSPACE_KEY},
    dependency_declaration::DependencyDeclaration,
    dependency_kind::DependencyKind,
    version_declaration::VersionDeclaration,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct CargoManifest {
    path: String,
    package: Option<String>,
    version: Option<VersionDeclaration>,
    workspace_version: Option<semver::Version>,
    dependencies: Vec<DependencyDeclaration>,
    workspace_dependencies: Vec<DependencyDeclaration>,
}

impl CargoManifest {
    const FILENAME: &str = "Cargo.toml";

    pub(crate) fn matches_path(path: &str) -> bool {
        path.rsplit('/').next() == Some(Self::FILENAME)
    }

    pub(crate) fn parse(path: String, content: &str) -> Result<Self, Box<dyn Error>> {
        let document: toml::Table = toml::from_str(content)?;

        let package_document = document.get(PACKAGE_KEY);
        let workspace_document = document.get(WORKSPACE_KEY);

        let package = package_document
            .and_then(|value| value.get("name"))
            .and_then(toml::Value::as_str)
            .map(str::to_owned);

        let version = package_document
            .and_then(|value| value.get(VERSION_KEY))
            .map(VersionDeclaration::parse)
            .transpose()?;

        let workspace_version = workspace_document
            .and_then(|value| value.get(PACKAGE_KEY))
            .and_then(|value| value.get(VERSION_KEY))
            .and_then(toml::Value::as_str)
            .map(semver::Version::parse)
            .transpose()?;

        let mut dependencies = Vec::new();
        let mut workspace_dependencies = Vec::new();

        Self::collect_dependencies(&document, None, &mut dependencies)?;

        if let Some(targets) = document.get("target") {
            let targets = targets.as_table().ok_or("Invalid target table")?;

            for (target, configuration) in targets {
                let configuration = configuration
                    .as_table()
                    .ok_or("Invalid target configuration")?;

                Self::collect_dependencies(configuration, Some(target), &mut dependencies)?;
            }
        }

        if let Some(workspace) = workspace_document {
            let workspace = workspace.as_table().ok_or("Invalid workspace table")?;

            Self::collect_dependencies(workspace, None, &mut workspace_dependencies)?;
        }

        Ok(Self {
            path,
            package,
            version,
            workspace_version,
            dependencies,
            workspace_dependencies,
        })
    }

    fn collect_dependencies(
        document: &toml::Table,
        target: Option<&str>,
        dependencies: &mut Vec<DependencyDeclaration>,
    ) -> Result<(), Box<dyn Error>> {
        for kind in DependencyKind::ALL {
            let key = kind.manifest_key();

            let Some(value) = document.get(key) else {
                continue;
            };

            let table = value
                .as_table()
                .ok_or_else(|| format!("Invalid dependency table: {key}"))?;

            for (name, value) in table {
                dependencies.push(DependencyDeclaration::from_manifest(
                    name, value, kind, target,
                )?);
            }
        }

        Ok(())
    }
}
