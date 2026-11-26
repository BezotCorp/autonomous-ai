use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::acquisition::{
    constants::{PACKAGE_KEY, VERSION_KEY, WORKSPACE_KEY},
    dependency_kind::DependencyKind,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct DependencyDeclaration {
    name: String,
    package: Option<String>,
    requirement: Option<semver::VersionReq>,
    kind: DependencyKind,
    target: Option<String>,
    workspace: bool,
    optional: Option<bool>,
    default_features: Option<bool>,
    features: Vec<String>,
    path: Option<String>,
    registry: Option<String>,
    registry_index: Option<String>,
    git: Option<String>,
    branch: Option<String>,
    tag: Option<String>,
    revision: Option<String>,
}

impl DependencyDeclaration {
    /// Name of the depended-upon package, which may differ from the alias.
    pub(crate) fn package(&self) -> &str {
        self.package.as_deref().unwrap_or(&self.name)
    }

    pub(crate) fn requirement(&self) -> Option<&semver::VersionReq> {
        self.requirement.as_ref()
    }

    pub(crate) fn kind(&self) -> DependencyKind {
        self.kind
    }

    pub(crate) fn is_inherited(&self) -> bool {
        self.workspace
    }

    pub(crate) fn from_manifest(
        name: &str,
        value: &toml::Value,
        kind: DependencyKind,
        target: Option<&str>,
    ) -> Result<Self, Box<dyn Error>> {
        let declaration = match value {
            toml::Value::String(requirement) => {
                let mut table = toml::Table::new();

                table.insert(
                    VERSION_KEY.to_owned(),
                    toml::Value::String(requirement.clone()),
                );

                table
            }

            toml::Value::Table(table) => table.clone(),

            _ => {
                return Err(format!("Invalid dependency declaration: {name}").into());
            }
        };

        let string = |key: &str| -> Result<Option<String>, Box<dyn Error>> {
            declaration
                .get(key)
                .map(|value| {
                    value.as_str().map(str::to_owned).ok_or_else(|| {
                        format!("Invalid string field {key} in dependency {name}").into()
                    })
                })
                .transpose()
        };

        let boolean = |key: &str| -> Result<Option<bool>, Box<dyn Error>> {
            declaration
                .get(key)
                .map(|value| {
                    value.as_bool().ok_or_else(|| {
                        format!("Invalid boolean field {key} in dependency {name}").into()
                    })
                })
                .transpose()
        };

        let features = declaration
            .get("features")
            .cloned()
            .map(toml::Value::try_into)
            .transpose()?
            .unwrap_or_default();

        let requirement = string(VERSION_KEY)?
            .as_deref()
            .map(semver::VersionReq::parse)
            .transpose()?;

        Ok(Self {
            name: name.to_owned(),
            package: string(PACKAGE_KEY)?,
            requirement,
            kind,
            target: target.map(str::to_owned),
            workspace: boolean(WORKSPACE_KEY)?.unwrap_or(false),
            optional: boolean("optional")?,
            default_features: boolean("default-features")?,
            features,
            path: string("path")?,
            registry: string("registry")?,
            registry_index: string("registry-index")?,
            git: string("git")?,
            branch: string("branch")?,
            tag: string("tag")?,
            revision: string("rev")?,
        })
    }
}
