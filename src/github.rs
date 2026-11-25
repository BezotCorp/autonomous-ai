use std::{error::Error, process::Command};

use serde_json::Value;

use crate::repository::Repository;

pub(crate) struct Github;

impl Github {
    fn api(endpoint: &str) -> Result<String, Box<dyn Error>> {
        let output = Command::new("gh").args(["api", endpoint]).output()?;

        if !output.status.success() {
            return Err(format!(
                "GitHub API request failed for {endpoint}: {}",
                String::from_utf8_lossy(&output.stderr),
            )
            .into());
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    pub(crate) fn revision(
        repository: &Repository,
        reference: &str,
    ) -> Result<String, Box<dyn Error>> {
        let content = Self::api(&format!(
            "repos/{}/commits/{}",
            repository.full_name(),
            reference,
        ))?;

        let response: Value = serde_json::from_str(&content)?;

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
        let content = Self::api(&format!(
            "repos/{}/git/trees/{}?recursive=1",
            repository.full_name(),
            revision,
        ))?;

        let response: Value = serde_json::from_str(&content)?;

        if response.get("truncated").and_then(Value::as_bool) != Some(false) {
            return Err("GitHub returned an incomplete tree".into());
        }

        let tree = response
            .get("tree")
            .and_then(Value::as_array)
            .ok_or("Missing GitHub tree")?;

        Ok(tree
            .iter()
            .filter(|entry| entry.get("type").and_then(Value::as_str) == Some("blob"))
            .filter_map(|entry| entry.get("path").and_then(Value::as_str))
            .map(str::to_owned)
            .collect())
    }

    pub(crate) fn read_file(
        repository: &Repository,
        revision: &str,
        path: &str,
    ) -> Result<String, Box<dyn Error>> {
        let output = Command::new("gh")
            .args([
                "api",
                "--method",
                "GET",
                "-H",
                "Accept: application/vnd.github.raw+json",
                &format!("repos/{}/contents/{}", repository.full_name(), path,),
                "-f",
                &format!("ref={revision}"),
            ])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Unable to read {path}: {}",
                String::from_utf8_lossy(&output.stderr),
            )
            .into());
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    pub(crate) fn repositories(owner: &str) -> Result<Vec<Repository>, Box<dyn Error>> {
        let output = Command::new("gh")
            .args([
                "api",
                "--paginate",
                "--slurp",
                &format!("orgs/{owner}/repos?per_page=100&type=all"),
            ])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Unable to enumerate repositories: {}",
                String::from_utf8_lossy(&output.stderr),
            )
            .into());
        }

        let pages: Vec<Vec<Value>> = serde_json::from_slice(&output.stdout)?;

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
