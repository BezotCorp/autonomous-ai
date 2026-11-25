use std::{
    error::Error,
    sync::atomic::{AtomicUsize, Ordering},
    thread,
};

use crate::{
    cargo_acquisition::CargoAcquisition, cargo_lockfile::CargoLockfile,
    cargo_manifest::CargoManifest, cargo_snapshot::CargoSnapshot, github::Github,
    knowledge::Knowledge, repository::Repository, snapshot_identity::SnapshotIdentity,
};

pub(crate) struct CargoObserver;

impl CargoObserver {
    const MAX_CONCURRENT_ACQUISITIONS: usize = 4;

    pub(crate) fn acquire(
        knowledge: &Knowledge,
        repository: &Repository,
        reference: &str,
    ) -> Result<CargoAcquisition, Box<dyn Error>> {
        let revision = Github::revision(repository, reference)?;

        let identity = SnapshotIdentity::new(repository.clone(), revision.clone());

        let snapshot = if knowledge.snapshot(&identity).is_some() {
            None
        } else {
            Some(Self::observe(repository, &revision)?)
        };

        Ok(CargoAcquisition::new(identity, snapshot))
    }

    pub(crate) fn acquire_many(
        knowledge: &Knowledge,
        repositories: &[Repository],
        reference: &str,
    ) -> Vec<(Repository, Result<CargoAcquisition, String>)> {
        let worker_count = repositories.len().min(Self::MAX_CONCURRENT_ACQUISITIONS);

        let next = AtomicUsize::new(0);

        let mut results = thread::scope(|scope| {
            let mut workers = Vec::with_capacity(worker_count);

            for _ in 0..worker_count {
                workers.push(scope.spawn(|| {
                    let mut results = Vec::new();

                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);

                        let Some(repository) = repositories.get(index) else {
                            break;
                        };

                        let result = Self::acquire(knowledge, repository, reference)
                            .map_err(|error| error.to_string());

                        results.push((repository.clone(), result));
                    }

                    results
                }));
            }

            workers
                .into_iter()
                .flat_map(|worker| match worker.join() {
                    Ok(results) => results,
                    Err(payload) => std::panic::resume_unwind(payload),
                })
                .collect::<Vec<_>>()
        });

        results.sort_by(|left, right| left.0.cmp(&right.0));

        results
    }

    fn observe(repository: &Repository, revision: &str) -> Result<CargoSnapshot, Box<dyn Error>> {
        let files = Github::files(repository, revision)?;

        let mut manifests = Vec::new();
        let mut lockfiles = Vec::new();

        for path in files {
            if CargoManifest::matches_path(&path) {
                let content = Github::read_file(repository, revision, &path)?;

                manifests.push(CargoManifest::parse(path, &content)?);
            } else if CargoLockfile::matches_path(&path) {
                let content = Github::read_file(repository, revision, &path)?;

                lockfiles.push(CargoLockfile::parse(path, &content)?);
            }
        }

        Ok(CargoSnapshot::new(manifests, lockfiles))
    }
}
