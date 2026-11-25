use std::{collections::BTreeSet, error::Error};

use semver::Version;
use serde::Serialize;

use crate::{
    cargo_lockfile::CargoLockfile, cargo_manifest::CargoManifest, github::Github,
    repository::Repository, rust_surface::RustSurface, source_difference::SourceDifference,
};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ReleaseComparison {
    previous_revision: String,
    current_revision: String,
    changed_files: Vec<String>,
    differences: Vec<SourceDifference>,
    facts: BTreeSet<String>,
}

impl ReleaseComparison {
    pub(crate) fn relation_facts(&self) -> Vec<crate::relation_fact::RelationFact> {
        use crate::relation_fact::RelationFact;

        let mut facts = BTreeSet::new();

        facts.insert(RelationFact::new(
            "release_revision_transition",
            vec![
                self.previous_revision.clone(),
                self.current_revision.clone(),
            ],
        ));

        for difference in &self.differences {
            facts.insert(RelationFact::new(
                "source_change",
                vec![
                    difference.path().to_owned(),
                    difference.category().to_owned(),
                ],
            ));

            for item in difference.previous() {
                facts.insert(RelationFact::new(
                    "source_item_removed",
                    vec![
                        difference.path().to_owned(),
                        difference.category().to_owned(),
                        item.clone(),
                    ],
                ));
            }

            for item in difference.current() {
                facts.insert(RelationFact::new(
                    "source_item_added",
                    vec![
                        difference.path().to_owned(),
                        difference.category().to_owned(),
                        item.clone(),
                    ],
                ));
            }
        }

        facts.into_iter().collect()
    }

    pub(crate) fn previous_revision(&self) -> &str {
        &self.previous_revision
    }

    pub(crate) fn current_revision(&self) -> &str {
        &self.current_revision
    }

    pub(crate) fn facts(&self) -> &BTreeSet<String> {
        &self.facts
    }

    pub(crate) fn acquire(
        repository: &Repository,
        previous: &Version,
        current: &Version,
    ) -> Result<Self, Box<dyn Error>> {
        let previous_revision = Github::release_revision(repository, previous)?;

        let current_revision = Github::release_revision(repository, current)?;

        let previous_files = Github::file_blobs(repository, &previous_revision)?;

        let current_files = Github::file_blobs(repository, &current_revision)?;

        let paths: BTreeSet<_> = previous_files.keys().chain(current_files.keys()).collect();

        let mut changed_files = Vec::new();
        let mut differences = Vec::new();
        let mut facts = BTreeSet::new();

        if previous.major != current.major {
            facts.insert("major_version_changed".to_owned());
        } else if previous.minor != current.minor {
            facts.insert("minor_version_changed".to_owned());
        } else if previous.patch != current.patch {
            facts.insert("patch_version_changed".to_owned());
        } else {
            facts.insert("version_qualifier_changed".to_owned());
        }

        if previous.major == 0 || current.major == 0 {
            facts.insert("unstable_version".to_owned());
        } else if previous.pre.is_empty() && current.pre.is_empty() {
            facts.insert("stable_version_contract".to_owned());
        }

        for path in paths {
            if previous_files.get(path) == current_files.get(path) {
                continue;
            }

            changed_files.push(path.clone());

            if path.ends_with(".rs") {
                let before = if previous_files.contains_key(path) {
                    RustSurface::parse(&Github::read_file(repository, &previous_revision, path)?)?
                } else {
                    RustSurface::default()
                };

                let after = if current_files.contains_key(path) {
                    RustSurface::parse(&Github::read_file(repository, &current_revision, path)?)?
                } else {
                    RustSurface::default()
                };

                differences.extend(before.differences(&after, path));

                if previous_files.contains_key(path) != current_files.contains_key(path) {
                    facts.insert("rust_file_layout_changed".to_owned());
                }
            } else if CargoManifest::matches_path(path) {
                facts.insert("cargo_manifest_changed".to_owned());
            } else if CargoLockfile::matches_path(path) {
                facts.insert("cargo_lockfile_changed".to_owned());
            } else {
                facts.insert("other_files_changed".to_owned());
            }
        }

        for difference in &differences {
            facts.insert(format!("{}_changed", difference.category(),));
        }

        for category in ["data_declaration", "serialization_impl"] {
            if !differences.iter().any(|item| item.category() == category) {
                facts.insert(format!("{category}_unchanged_in_observed_source",));
            }
        }

        if differences.is_empty() && !facts.contains("rust_file_layout_changed") {
            facts.insert("rust_syntax_unchanged".to_owned());
        }

        facts.insert("release_comparison_available".to_owned());

        Ok(Self {
            previous_revision,
            current_revision,
            changed_files,
            differences,
            facts,
        })
    }
}
