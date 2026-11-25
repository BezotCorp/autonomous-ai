use std::{error::Error, process::Command};

use serde_json::Value;

use crate::repository::Repository;

pub(crate) struct Github;

impl Github {
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

        let mut files = Vec::new();

        for entry in tree {
            if entry.get("type").and_then(Value::as_str) != Some("blob") {
                continue;
            }

            let path = entry
                .get("path")
                .and_then(Value::as_str)
                .ok_or("Missing GitHub file path")?;

            files.push(path.to_owned());
        }

        Ok(files)
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
