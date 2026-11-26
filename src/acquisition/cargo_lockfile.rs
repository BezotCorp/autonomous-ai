use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::acquisition::{
    constants::{PACKAGE_KEY, VERSION_KEY},
    resolved_package::ResolvedPackage,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct CargoLockfile {
    path: String,
    version: Option<u32>,
    packages: Vec<ResolvedPackage>,
}

impl CargoLockfile {
    const FILENAME: &str = "Cargo.lock";

    pub(crate) fn matches_path(path: &str) -> bool {
        path.rsplit('/').next() == Some(Self::FILENAME)
    }

    pub(crate) fn parse(path: String, content: &str) -> Result<Self, Box<dyn Error>> {
        let mut document: toml::Table = toml::from_str(content)?;

        let version = document
            .remove(VERSION_KEY)
            .map(toml::Value::try_into)
            .transpose()?;

        let mut packages: Vec<ResolvedPackage> = document
            .remove(PACKAGE_KEY)
            .map(toml::Value::try_into)
            .transpose()?
            .unwrap_or_default();

        if let Some(root) = document.remove("root") {
            packages.push(root.try_into()?);
        }

        Ok(Self {
            path,
            version,
            packages,
        })
    }

    pub(crate) fn path(&self) -> &str {
        &self.path
    }

    pub(crate) fn packages(&self) -> &[ResolvedPackage] {
        &self.packages
    }
}
