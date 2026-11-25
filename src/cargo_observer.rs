use std::error::Error;

use crate::{
    cargo_manifest::CargoManifest, github::Github, repository::Repository,
    repository_snapshot::RepositorySnapshot,
};

pub(crate) struct CargoObserver;

impl CargoObserver {
    pub(crate) fn observe(
        repository: &Repository,
        reference: &str,
    ) -> Result<RepositorySnapshot, Box<dyn Error>> {
        let revision = Github::revision(repository, reference)?;
        let files = Github::files(repository, &revision)?;

        let mut manifests = Vec::new();
        let mut lockfiles = Vec::new();

        for path in files {
            let filename = path.rsplit('/').next();

            match filename {
                Some("Cargo.toml") => {
                    let content = Github::read_file(repository, &revision, &path)?;

                    manifests.push(CargoManifest::parse(path, &content)?);
                }
                Some("Cargo.lock") => {
                    let content = Github::read_file(repository, &revision, &path)?;

                    let _: toml::Value = toml::from_str(&content)?;

                    lockfiles.push((path, content));
                }
                _ => {}
            }
        }

        Ok(RepositorySnapshot::new(
            repository.clone(),
            revision,
            manifests,
            lockfiles,
        ))
    }
}
