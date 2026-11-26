use std::{error::Error, process::Command};

use serde_json::Value;

use crate::acquisition::repository::Repository;

pub(crate) struct Github;

impl Github {
    pub(crate) fn release_revision(
        repository: &Repository,
        version: &semver::Version,
    ) -> Result<String, Box<dyn Error>> {
        let mut errors = Vec::new();

        for tag in [version.to_string(), format!("v{version}")] {
            let endpoint = format!(
                "{}/git/ref/tags/{}",
                Self::repository_endpoint(repository),
                Self::encode_segment(&tag),
            );

            let response = match Self::api(&[&endpoint]) {
                Ok(response) => response,
                Err(error) => {
                    errors.push(error.to_string());
                    continue;
                }
            };

            let response: Value = serde_json::from_str(&response)?;

            let mut object = response
                .get("object")
                .cloned()
                .ok_or("Missing Git tag object")?;

            loop {
                let sha = object
                    .get("sha")
                    .and_then(Value::as_str)
                    .ok_or("Missing Git object SHA")?;

                match object.get("type").and_then(Value::as_str) {
                    Some("commit") => return Ok(sha.to_owned()),

                    Some("tag") => {
                        let endpoint = format!(
                            "{}/git/tags/{}",
                            Self::repository_endpoint(repository),
                            Self::encode_segment(sha),
                        );

                        let response: Value = serde_json::from_str(&Self::api(&[&endpoint])?)?;

                        object = response
                            .get("object")
                            .cloned()
                            .ok_or("Missing annotated tag object")?;
                    }

                    _ => return Err("Release tag does not resolve to a commit".into()),
                }
            }
        }

        Err(format!(
            "Unable to resolve release {} in {}: {}",
            version,
            repository.full_name(),
            errors.join("; "),
        )
        .into())
    }

    pub(crate) fn file_blobs(
        repository: &Repository,
        revision: &str,
    ) -> Result<std::collections::BTreeMap<String, String>, Box<dyn Error>> {
        let endpoint = format!(
            "{}/git/trees/{}?recursive=1",
            Self::repository_endpoint(repository),
            Self::encode_segment(revision),
        );

        let response: Value = serde_json::from_str(&Self::api(&[&endpoint])?)?;

        if response.get("truncated").and_then(Value::as_bool) != Some(false) {
            return Err("GitHub returned an incomplete tree".into());
        }

        let tree = response
            .get("tree")
            .and_then(Value::as_array)
            .ok_or("Missing GitHub tree")?;

        let mut files = std::collections::BTreeMap::new();

        for entry in tree {
            if entry.get("type").and_then(Value::as_str) != Some("blob") {
                continue;
            }

            let path = entry
                .get("path")
                .and_then(Value::as_str)
                .ok_or("Missing GitHub file path")?;

            let sha = entry
                .get("sha")
                .and_then(Value::as_str)
                .ok_or("Missing GitHub blob SHA")?;

            files.insert(path.to_owned(), sha.to_owned());
        }

        Ok(files)
    }

    fn api(arguments: &[&str]) -> Result<String, Box<dyn Error>> {
        let output = Command::new("gh")
            .args(["api", "--method", "GET"])
            .args(arguments)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "GitHub API request failed: {}",
                String::from_utf8_lossy(&output.stderr).trim(),
            )
            .into());
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    fn encode_segment(value: &str) -> String {
        let mut encoded = String::new();

        for byte in value.bytes() {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
                encoded.push(char::from(byte));
            } else {
                encoded.push_str(&format!("%{byte:02X}"));
            }
        }

        encoded
    }

    fn repository_endpoint(repository: &Repository) -> String {
        format!(
            "repos/{}/{}",
            Self::encode_segment(repository.owner()),
            Self::encode_segment(repository.name()),
        )
    }

    pub(crate) fn revision(
        repository: &Repository,
        reference: &str,
    ) -> Result<String, Box<dyn Error>> {
        let endpoint = format!(
            "{}/commits/{}",
            Self::repository_endpoint(repository),
            Self::encode_segment(reference),
        );

        let response: Value = serde_json::from_str(&Self::api(&[&endpoint])?)?;

        response
            .get("sha")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| "Missing GitHub commit SHA".into())
    }

    pub(crate) fn files(
        repository: &Repository,
        revision: &str,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(Self::file_blobs(repository, revision)?
            .into_keys()
            .collect())
    }

    pub(crate) fn read_file(
        repository: &Repository,
        revision: &str,
        path: &str,
    ) -> Result<String, Box<dyn Error>> {
        let path = path
            .split('/')
            .map(Self::encode_segment)
            .collect::<Vec<_>>()
            .join("/");

        let endpoint = format!(
            "{}/contents/{}",
            Self::repository_endpoint(repository),
            path,
        );

        Self::api(&[
            "-H",
            "Accept: application/vnd.github.raw+json",
            &endpoint,
            "-f",
            &format!("ref={revision}"),
        ])
    }

    pub(crate) fn repositories(owner: &str) -> Result<Vec<Repository>, Box<dyn Error>> {
        let endpoint = format!(
            "orgs/{}/repos?per_page=100&type=all",
            Self::encode_segment(owner),
        );

        let pages: Vec<Vec<Value>> =
            serde_json::from_str(&Self::api(&["--paginate", "--slurp", &endpoint])?)?;

        let mut repositories = Vec::new();

        for entry in pages.into_iter().flatten() {
            if entry.get("archived").and_then(Value::as_bool) == Some(true) {
                continue;
            }

            let name = entry
                .get("name")
                .and_then(Value::as_str)
                .ok_or("Missing repository name")?;

            repositories.push(Repository::new(owner, name));
        }

        repositories.sort();
        repositories.dedup();

        Ok(repositories)
    }
}
