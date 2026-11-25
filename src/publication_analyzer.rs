use std::error::Error;

use crate::{
    knowledge::Knowledge, older_resolved_package::OlderResolvedPackage,
    publication_analysis::PublicationAnalysis, registry::Registry,
    snapshot_identity::SnapshotIdentity,
};

pub(crate) struct PublicationAnalyzer;

impl PublicationAnalyzer {
    pub(crate) fn analyze(
        event_id: &str,
        knowledge: &Knowledge,
        identities: Vec<SnapshotIdentity>,
        name: &str,
        registry: &str,
        version: &semver::Version,
    ) -> Result<PublicationAnalysis, Box<dyn Error>> {
        let registry = Registry::from_index(registry);
        let mut older_packages = Vec::new();

        for identity in &identities {
            let snapshot = knowledge.snapshot(identity).ok_or_else(|| {
                format!("Missing snapshot for {}", identity.repository().full_name(),)
            })?;

            for lockfile in snapshot.lockfiles() {
                for package in lockfile.older_packages(name, &registry, version) {
                    older_packages.push(OlderResolvedPackage::new(
                        identity.clone(),
                        lockfile.path().to_owned(),
                        package,
                    ));
                }
            }
        }

        Ok(PublicationAnalysis::new(
            event_id.to_owned(),
            identities,
            older_packages,
        ))
    }
}
