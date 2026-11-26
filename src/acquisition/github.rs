use std::{collections::BTreeMap, process::Command};

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::{
    acquisition::{
        access_mode::AccessMode, external_access::ExternalAccess, http_response::HttpResponse,
        repository::Repository, response_store::ResponseStore,
    },
    consumables::{resource_meter::ResourceMeter, resource_state::ResourceState},
    memory::cached_response::CachedResponse,
};

/// Client of the GitHub API (through the `gh` command) that acquires
/// sparingly:
///
/// - data addressed by an immutable identifier (a commit, tree or blob SHA)
///   is acquired once and then read from the store;
/// - mutable data is revalidated with its entity tag: an unchanged answer
///   (304) consumes no quota;
/// - independent lookups are grouped into one GraphQL query when cheaper;
/// - every request is authorised and accounted by the resource meter, and
///   the quota the API reports is observed after each response.
///
/// In replay mode, nothing leaves the process: recorded data is served as
/// it was at the replayed instant, missing data stays missing, and the
/// consumption the acquisition would have had then is simulated.
pub(crate) struct Github<'a> {
    meter: &'a ResourceMeter,
    store: &'a ResponseStore<'a>,
    mode: AccessMode,
}

/// How long a stored response stays usable.
#[derive(Clone, Copy, PartialEq)]
enum Freshness {
    /// Identified by its content: valid forever.
    Immutable,
    /// May change: reused only if the source confirms it did not.
    Revalidated,
}

impl<'a> Github<'a> {
    pub(crate) const REST_RESOURCE: &'static str = "github:core";
    pub(crate) const GRAPHQL_RESOURCE: &'static str = "github:graphql";
    const API_ROOT: &'static str = "https://api.github.com/";
    const GRAPHQL_BATCH: usize = 50;
    const RAW: &'static str = "application/vnd.github.raw+json";
    const SHA: &'static str = "application/vnd.github.sha";

    pub(crate) fn new(access: &'a ExternalAccess<'a>) -> Self {
        Self {
            meter: access.meter(),
            store: access.store(),
            mode: access.mode(),
        }
    }

    /// Whether a reference is a full commit SHA, hence immutable.
    pub(crate) fn is_commit_sha(reference: &str) -> bool {
        reference.len() == 40 && reference.bytes().all(|byte| byte.is_ascii_hexdigit())
    }

    pub(crate) fn revision(
        &self,
        repository: &Repository,
        reference: &str,
    ) -> Result<String, String> {
        let endpoint = format!(
            "{}/commits/{}",
            Self::repository_endpoint(repository),
            Self::encode_segment(reference),
        );

        let freshness = if Self::is_commit_sha(reference) {
            Freshness::Immutable
        } else {
            Freshness::Revalidated
        };

        Ok(self
            .rest(&endpoint, Some(Self::SHA), freshness)?
            .trim()
            .to_owned())
    }

    /// Current revision of the default branch of each repository, looked
    /// up in grouped GraphQL queries.
    pub(crate) fn head_revisions(
        &self,
        repositories: &[Repository],
    ) -> BTreeMap<Repository, Result<String, String>> {
        let mut revisions = BTreeMap::new();

        if let AccessMode::Replay { as_of, .. } = self.mode {
            for batch in repositories.chunks(Self::GRAPHQL_BATCH) {
                // Revisions are always looked up afresh: one query per batch.
                if let Err(error) = self.meter.authorize(Self::GRAPHQL_RESOURCE, 1.0) {
                    for repository in batch {
                        revisions.insert(repository.clone(), Err(error.clone()));
                    }
                    continue;
                }

                self.meter.record(
                    Self::GRAPHQL_RESOURCE,
                    &format!("replayed heads {as_of}"),
                    1.0,
                );

                for repository in batch {
                    let revision = self
                        .store
                        .recorded(&Self::head_key(repository), as_of)
                        .map(|recorded| recorded.body().to_owned())
                        .ok_or_else(|| {
                            format!(
                                "revision of {} not recorded by {as_of}, left missing",
                                repository.full_name()
                            )
                        });

                    revisions.insert(repository.clone(), revision);
                }
            }

            return revisions;
        }

        for batch in repositories.chunks(Self::GRAPHQL_BATCH) {
            let fields: Vec<String> = batch
                .iter()
                .enumerate()
                .map(|(index, repository)| {
                    format!(
                        "r{index}: repository(owner: {}, name: {}) {{ defaultBranchRef {{ target {{ oid }} }} }}",
                        Self::literal(repository.owner()),
                        Self::literal(repository.name()),
                    )
                })
                .collect();

            match self.graphql(&fields.join(" ")) {
                Ok(data) => {
                    for (index, repository) in batch.iter().enumerate() {
                        let revision = data
                            .get(format!("r{index}"))
                            .and_then(|entry| entry.pointer("/defaultBranchRef/target/oid"))
                            .and_then(Value::as_str)
                            .map(str::to_owned)
                            .ok_or_else(|| {
                                format!("no default branch revision for {}", repository.full_name())
                            });

                        if let Ok(revision) = &revision {
                            self.store.put(
                                &Self::head_key(repository),
                                CachedResponse::mutable(revision.clone(), None),
                            );
                        }

                        revisions.insert(repository.clone(), revision);
                    }
                }
                Err(error) => {
                    for repository in batch {
                        revisions.insert(repository.clone(), Err(error.clone()));
                    }
                }
            }
        }

        revisions
    }

    pub(crate) fn release_revision(
        &self,
        repository: &Repository,
        version: &semver::Version,
    ) -> Result<String, String> {
        let mut errors = Vec::new();

        for tag in [version.to_string(), format!("v{version}")] {
            let endpoint = format!(
                "{}/git/ref/tags/{}",
                Self::repository_endpoint(repository),
                Self::encode_segment(&tag),
            );

            let response = match self.rest(&endpoint, None, Freshness::Revalidated) {
                Ok(response) => response,
                Err(error) => {
                    errors.push(error);
                    continue;
                }
            };

            let response: Value =
                serde_json::from_str(&response).map_err(|error| error.to_string())?;

            let mut object = response
                .get("object")
                .cloned()
                .ok_or("Missing Git tag object")?;

            loop {
                let sha = object
                    .get("sha")
                    .and_then(Value::as_str)
                    .ok_or("Missing Git object SHA")?
                    .to_owned();

                match object.get("type").and_then(Value::as_str) {
                    Some("commit") => return Ok(sha),

                    Some("tag") => {
                        let endpoint = format!(
                            "{}/git/tags/{}",
                            Self::repository_endpoint(repository),
                            Self::encode_segment(&sha),
                        );

                        let response: Value = serde_json::from_str(&self.rest(
                            &endpoint,
                            None,
                            Freshness::Immutable,
                        )?)
                        .map_err(|error| error.to_string())?;

                        object = response
                            .get("object")
                            .cloned()
                            .ok_or("Missing annotated tag object")?;
                    }

                    _ => return Err("Release tag does not resolve to a commit".to_owned()),
                }
            }
        }

        Err(format!(
            "Unable to resolve release {} in {}: {}",
            version,
            repository.full_name(),
            errors.join("; "),
        ))
    }

    /// Path and blob SHA of every file at a revision.
    pub(crate) fn file_blobs(
        &self,
        repository: &Repository,
        revision: &str,
    ) -> Result<BTreeMap<String, String>, String> {
        let endpoint = format!(
            "{}/git/trees/{}?recursive=1",
            Self::repository_endpoint(repository),
            Self::encode_segment(revision),
        );

        let freshness = if Self::is_commit_sha(revision) {
            Freshness::Immutable
        } else {
            Freshness::Revalidated
        };

        let response: Value = serde_json::from_str(&self.rest(&endpoint, None, freshness)?)
            .map_err(|error| error.to_string())?;

        if response.get("truncated").and_then(Value::as_bool) != Some(false) {
            return Err("GitHub returned an incomplete tree".to_owned());
        }

        let tree = response
            .get("tree")
            .and_then(Value::as_array)
            .ok_or("Missing GitHub tree")?;

        let mut files = BTreeMap::new();

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

    /// Contents of blobs, identified by their SHA. Stored blobs are reused;
    /// the missing ones are fetched in grouped GraphQL queries or one by
    /// one, whichever the resources currently make cheaper.
    pub(crate) fn blobs(
        &self,
        repository: &Repository,
        shas: &[String],
    ) -> BTreeMap<String, Result<String, String>> {
        let mut contents = BTreeMap::new();
        let mut missing = Vec::new();

        if let AccessMode::Replay { as_of, since } = self.mode {
            let mut acquired: usize = 0;

            for sha in shas {
                let content = match self.store.recorded(&Self::blob_key(sha), as_of) {
                    Some(recorded) => {
                        if recorded.stored_at() < since {
                            self.meter.avoid();
                        } else {
                            acquired += 1;
                        }

                        Ok(recorded.body().to_owned())
                    }
                    None => Err(format!("blob {sha} not recorded, left missing")),
                };

                contents.insert(sha.clone(), content);
            }

            if acquired > 0 {
                let batches = acquired.div_ceil(Self::GRAPHQL_BATCH) as f64;

                if self
                    .meter
                    .authorize(Self::GRAPHQL_RESOURCE, batches)
                    .is_err()
                {
                    return shas
                        .iter()
                        .map(|sha| (sha.clone(), Err("resource exhausted in replay".to_owned())))
                        .collect();
                }

                self.meter.record(
                    Self::GRAPHQL_RESOURCE,
                    &format!("replayed blobs {as_of}"),
                    batches,
                );
            }

            return contents;
        }

        for sha in shas {
            if contents.contains_key(sha) || missing.contains(sha) {
                continue;
            }

            match self.store.get(&Self::blob_key(sha)) {
                Some(cached) => {
                    self.meter.avoid();
                    contents.insert(sha.clone(), Ok(cached.body().to_owned()));
                }
                None => missing.push(sha.clone()),
            }
        }

        if missing.is_empty() {
            return contents;
        }

        let batches = missing.len().div_ceil(Self::GRAPHQL_BATCH);
        let alternatives = [
            (
                vec![(Self::GRAPHQL_RESOURCE.to_owned(), batches as f64)],
                batches,
            ),
            (
                vec![(Self::REST_RESOURCE.to_owned(), missing.len() as f64)],
                missing.len(),
            ),
        ];

        let mut remaining = missing.clone();

        if self.meter.cheapest(&alternatives) == Some(0) {
            remaining.clear();

            for batch in missing.chunks(Self::GRAPHQL_BATCH) {
                let fields: Vec<String> = batch
                    .iter()
                    .enumerate()
                    .map(|(index, sha)| {
                        format!(
                            "b{index}: object(oid: {}) {{ ... on Blob {{ text isTruncated isBinary }} }}",
                            Self::literal(sha)
                        )
                    })
                    .collect();

                let query = format!(
                    "repository(owner: {}, name: {}) {{ {} }}",
                    Self::literal(repository.owner()),
                    Self::literal(repository.name()),
                    fields.join(" "),
                );

                let data = match self.graphql(&query) {
                    Ok(data) => data,
                    Err(_) => {
                        remaining.extend_from_slice(batch);
                        continue;
                    }
                };

                for (index, sha) in batch.iter().enumerate() {
                    let blob = data.pointer(&format!("/repository/b{index}"));
                    let complete = blob.is_some_and(|blob| {
                        blob.get("isTruncated").and_then(Value::as_bool) == Some(false)
                            && blob.get("isBinary").and_then(Value::as_bool) == Some(false)
                    });

                    match blob
                        .and_then(|blob| blob.get("text"))
                        .and_then(Value::as_str)
                    {
                        Some(text) if complete => {
                            self.store.put(
                                &Self::blob_key(sha),
                                CachedResponse::immutable(text.to_owned()),
                            );
                            contents.insert(sha.clone(), Ok(text.to_owned()));
                        }
                        _ => remaining.push(sha.clone()),
                    }
                }
            }
        }

        for sha in remaining {
            let endpoint = format!(
                "{}/git/blobs/{}",
                Self::repository_endpoint(repository),
                Self::encode_segment(&sha),
            );

            let content = self.rest(&endpoint, Some(Self::RAW), Freshness::Immutable);

            if let Ok(text) = &content {
                self.store.put(
                    &Self::blob_key(&sha),
                    CachedResponse::immutable(text.clone()),
                );
            }

            contents.insert(sha, content);
        }

        contents
    }

    /// Non-archived repositories of an owner; unchanged pages are
    /// confirmed by their entity tag without consuming quota.
    pub(crate) fn repositories(&self, owner: &str) -> Result<Vec<Repository>, String> {
        let mut endpoint = format!(
            "orgs/{}/repos?per_page=100&type=all",
            Self::encode_segment(owner),
        );
        let mut repositories = Vec::new();

        loop {
            let (body, next) = self.rest_page(&endpoint, None, Freshness::Revalidated)?;
            let entries: Vec<Value> =
                serde_json::from_str(&body).map_err(|error| error.to_string())?;

            for entry in entries {
                if entry.get("archived").and_then(Value::as_bool) == Some(true) {
                    continue;
                }

                let name = entry
                    .get("name")
                    .and_then(Value::as_str)
                    .ok_or("Missing repository name")?;

                repositories.push(Repository::new(owner, name));
            }

            match next {
                Some(next) => endpoint = next,
                None => break,
            }
        }

        repositories.sort();
        repositories.dedup();

        Ok(repositories)
    }

    fn rest(
        &self,
        endpoint: &str,
        accept: Option<&str>,
        freshness: Freshness,
    ) -> Result<String, String> {
        self.rest_page(endpoint, accept, freshness)
            .map(|(body, _)| body)
    }

    /// One REST request, with its next page when the answer is paginated.
    fn rest_page(
        &self,
        endpoint: &str,
        accept: Option<&str>,
        freshness: Freshness,
    ) -> Result<(String, Option<String>), String> {
        let key = format!("github:{endpoint}|{}", accept.unwrap_or_default());

        if let AccessMode::Replay { as_of, since } = self.mode {
            let recorded = self.replayed(&key, Self::REST_RESOURCE, as_of, since)?;

            return Ok((
                recorded.body().to_owned(),
                recorded.next().map(str::to_owned),
            ));
        }

        let cached = self.store.get(&key);

        if freshness == Freshness::Immutable
            && let Some(cached) = &cached
        {
            self.meter.avoid();
            return Ok((cached.body().to_owned(), None));
        }

        self.meter.authorize(Self::REST_RESOURCE, 1.0)?;

        let mut arguments = vec!["api", "-i", "--method", "GET"];
        let accept_header = accept.map(|accept| format!("Accept: {accept}"));
        let validator_header = cached
            .as_ref()
            .and_then(CachedResponse::validator)
            .map(|validator| format!("If-None-Match: {validator}"));

        for header in accept_header.iter().chain(&validator_header) {
            arguments.extend(["-H", header]);
        }

        arguments.push(endpoint);

        let response = Self::execute(&arguments)?;
        let resource = Self::quota_resource(&response, Self::REST_RESOURCE);
        let units = if response.status() == 304 { 0.0 } else { 1.0 };

        self.meter.record(&resource, &key, units);
        self.observe_quota(&response, &resource);

        let next = response
            .next_page()
            .map(|next| next.trim_start_matches(Self::API_ROOT).to_owned());

        match response.status() {
            304 => {
                self.meter.avoid();

                let cached = cached.ok_or("Unexpected 304 response without stored data")?;
                Ok((cached.body().to_owned(), next))
            }
            200..=299 => {
                let validator = match freshness {
                    Freshness::Immutable => None,
                    Freshness::Revalidated => response.header("etag").map(str::to_owned),
                };

                let stored = match freshness {
                    Freshness::Immutable => CachedResponse::immutable(response.body().to_owned()),
                    Freshness::Revalidated => {
                        CachedResponse::mutable(response.body().to_owned(), validator)
                    }
                };

                self.store.put(&key, stored.with_next(next.clone()));

                Ok((response.into_body(), next))
            }
            status => Err(format!(
                "GitHub API request {endpoint} failed with status {status}: {}",
                Self::message(response.body())
            )),
        }
    }

    /// Recorded answer to a request, charging the meter with what the
    /// request would have consumed at the replayed instant.
    fn replayed(
        &self,
        key: &str,
        resource: &str,
        as_of: DateTime<Utc>,
        since: DateTime<Utc>,
    ) -> Result<CachedResponse, String> {
        let recorded = self
            .store
            .recorded(key, as_of)
            .ok_or_else(|| format!("not recorded by {as_of}, left missing: {key}"))?;

        match (recorded.stored_at() < since, recorded.is_immutable()) {
            (true, true) => self.meter.avoid(),
            (true, false) => {
                self.meter.authorize(resource, 1.0)?;
                self.meter.record(resource, key, 0.0);
                self.meter.avoid();
            }
            (false, _) => {
                self.meter.authorize(resource, 1.0)?;
                self.meter.record(resource, key, 1.0);
            }
        }

        Ok(recorded)
    }

    fn head_key(repository: &Repository) -> String {
        format!("github:head:{}", repository.full_name())
    }

    /// Runs a GraphQL query (the fields of the root `query`) and returns
    /// its data, which may be partial: absent entries are reported by the
    /// callers for what they concern.
    fn graphql(&self, fields: &str) -> Result<Value, String> {
        if self.mode != AccessMode::Live {
            return Err("no external request in replay".to_owned());
        }

        self.meter.authorize(Self::GRAPHQL_RESOURCE, 1.0)?;

        let query = format!("query {{ rateLimit {{ cost }} {fields} }}");
        let argument = format!("query={query}");
        let response = Self::execute(&["api", "-i", "graphql", "-f", &argument])?;
        let resource = Self::quota_resource(&response, Self::GRAPHQL_RESOURCE);
        let body: Value = serde_json::from_str(response.body()).map_err(|error| {
            format!(
                "Invalid GraphQL response (status {}): {error}",
                response.status()
            )
        })?;

        let cost = body
            .pointer("/data/rateLimit/cost")
            .and_then(Value::as_f64)
            .unwrap_or(1.0);

        self.meter.record(&resource, &query, cost);
        self.observe_quota(&response, &resource);

        match body.get("data") {
            Some(data) if !data.is_null() => Ok(data.clone()),
            _ => Err(format!(
                "GraphQL query failed with status {}: {}",
                response.status(),
                Self::message(response.body())
            )),
        }
    }

    /// Name of the resource whose quota the response consumed.
    fn quota_resource(response: &HttpResponse, default: &str) -> String {
        response
            .header("x-ratelimit-resource")
            .map(|name| format!("github:{name}"))
            .unwrap_or_else(|| default.to_owned())
    }

    /// Records the quota the response reports, which already accounts for
    /// the request itself.
    fn observe_quota(&self, response: &HttpResponse, resource: &str) {
        let number = |name: &str| {
            response
                .header(name)
                .and_then(|value| value.parse::<f64>().ok())
        };

        if let (Some(limit), Some(remaining)) =
            (number("x-ratelimit-limit"), number("x-ratelimit-remaining"))
        {
            let reset = number("x-ratelimit-reset")
                .and_then(|seconds| DateTime::<Utc>::from_timestamp(seconds as i64, 0));

            self.meter
                .observe(resource, ResourceState::limited(limit, remaining, reset));
        }
    }

    /// Only reached in live mode: every request leaving the process goes
    /// through here.
    fn execute(arguments: &[&str]) -> Result<HttpResponse, String> {
        let output = Command::new("gh")
            .args(arguments)
            .output()
            .map_err(|error| format!("Unable to run gh: {error}"))?;
        let stdout = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;

        // `gh` exits with an error status on 304 and partial GraphQL
        // answers: the HTTP response decides, when there is one.
        if stdout.starts_with("HTTP/") {
            return HttpResponse::parse(&stdout);
        }

        Err(format!(
            "GitHub API request failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }

    fn message(body: &str) -> String {
        serde_json::from_str::<Value>(body)
            .ok()
            .and_then(|value| {
                value
                    .get("message")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .or_else(|| value.get("errors").map(|errors| errors.to_string()))
            })
            .unwrap_or_else(|| body.chars().take(200).collect())
    }

    fn blob_key(sha: &str) -> String {
        format!("git-blob:{sha}")
    }

    fn literal(value: &str) -> String {
        serde_json::Value::String(value.to_owned()).to_string()
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
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use chrono::{Duration, Utc};

    use super::Github;
    use crate::{
        acquisition::external_access::ExternalAccess,
        consumables::resource_meter::ResourceMeter,
        memory::{acquisition_cache::AcquisitionCache, cached_response::CachedResponse},
    };

    #[test]
    fn replays_only_what_was_recorded_without_any_request() {
        let mut cache = AcquisitionCache::default();

        cache.integrate(
            BTreeMap::from([(
                "github:orgs/known/repos?per_page=100&type=all|".to_owned(),
                CachedResponse::mutable(r#"[{"name": "one", "archived": false}]"#.to_owned(), None),
            )]),
            BTreeSet::new(),
        );

        let now = Utc::now();
        let access = ExternalAccess::replay(
            ResourceMeter::default(),
            &cache,
            now,
            now - Duration::hours(1),
        );
        let github = Github::new(&access);

        assert_eq!(github.repositories("known").unwrap().len(), 1);
        assert!(
            github
                .repositories("unknown")
                .unwrap_err()
                .contains("left missing")
        );

        // Recorded during the replayed period: it would have cost a request.
        assert_eq!(
            access.meter().usage().requests().get(Github::REST_RESOURCE),
            Some(&1)
        );
    }
}
