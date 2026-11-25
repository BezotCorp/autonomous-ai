use std::collections::BTreeMap;

use semver::{Version, VersionReq};
use toml::Value;

use crate::{
    cargo_lockfile::CargoLockfile, cargo_manifest::CargoManifest,
    dependency_reference::DependencyReference, dependency_scan::DependencyScan, github::Github,
    repository::Repository, source_tokens::SourceTokens,
};

#[derive(Default)]
pub(crate) struct DependencyScanner {
    cache: BTreeMap<(Repository, String), DependencyScan>,
}

impl DependencyScanner {
    const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

    pub(crate) fn acquire(&mut self, repository: &Repository, revision: &str) -> &DependencyScan {
        self.cache
            .entry((repository.clone(), revision.to_owned()))
            .or_insert_with(|| Self::scan(repository, revision))
    }

    fn scan(repository: &Repository, revision: &str) -> DependencyScan {
        let mut scan = DependencyScan::default();

        let paths = match Github::files(repository, revision) {
            Ok(paths) => paths,

            Err(error) => {
                scan.fail(format!(
                    "{}@{}: {}",
                    repository.full_name(),
                    revision,
                    error,
                ));

                return scan;
            }
        };

        let mut manifests = BTreeMap::new();
        let mut lockfiles = BTreeMap::new();

        for path in &paths {
            let destination = if CargoManifest::matches_path(path) {
                &mut manifests
            } else if CargoLockfile::matches_path(path) {
                &mut lockfiles
            } else {
                continue;
            };

            let source = match Github::read_file(repository, revision, path) {
                Ok(source) => source,

                Err(error) => {
                    scan.fail(format!("{path}: {error}"));
                    continue;
                }
            };

            match toml::from_str::<Value>(&source) {
                Ok(document) => {
                    destination.insert(path.clone(), document);
                }

                Err(error) => {
                    scan.fail(format!("{path}: {error}"));
                }
            }
        }

        for path in paths.iter().filter(|path| path.ends_with(".rs")) {
            let Some((manifest_path, manifest)) = Self::nearest(&manifests, path, "Cargo.toml")
            else {
                scan.fail(format!("{path}: no enclosing Cargo manifest"));
                continue;
            };

            let Some(package) = manifest.get("package").and_then(Value::as_table) else {
                scan.fail(format!("{path}: no enclosing Cargo package"));
                continue;
            };

            let Some(owner_name) = package.get("name").and_then(Value::as_str) else {
                scan.fail(format!("{path}: missing Cargo package name"));
                continue;
            };

            let workspace = Self::workspace(&manifests, manifest_path);

            let owner_version = package
                .get("version")
                .and_then(Value::as_str)
                .or_else(|| workspace?.get("package")?.get("version")?.as_str());

            let Some((_, lockfile)) = Self::nearest(&lockfiles, manifest_path, "Cargo.lock") else {
                scan.fail(format!("{path}: no enclosing Cargo lockfile"));
                continue;
            };

            let Some(locked_packages) = lockfile.get("package").and_then(Value::as_array) else {
                scan.fail(format!("{path}: no locked packages"));
                continue;
            };

            let source = match Github::read_file(repository, revision, path) {
                Ok(source) => source,

                Err(error) => {
                    scan.fail(format!("{path}: {error}"));
                    continue;
                }
            };

            let tokens = match SourceTokens::parse(&source) {
                Ok(tokens) => tokens,

                Err(error) => {
                    scan.fail(format!("{path}: {error}"));
                    continue;
                }
            };

            let mut declarations = Vec::new();

            Self::collect_declarations(manifest, &mut declarations);

            if let Some(targets) = manifest.get("target").and_then(Value::as_table) {
                for configuration in targets.values() {
                    Self::collect_declarations(configuration, &mut declarations);
                }
            }

            for (alias, declaration) in declarations {
                let import_name = alias.replace('-', "_");

                if !tokens.references(&import_name) {
                    continue;
                }

                let declaration = if declaration.get("workspace").and_then(Value::as_bool)
                    == Some(true)
                {
                    let inherited = workspace
                        .and_then(|value| value.get("dependencies"))
                        .and_then(|value| value.get(alias));

                    match inherited {
                        Some(inherited) => inherited,

                        None => {
                            scan.fail(format!("{path}: unresolved inherited dependency {alias}",));

                            continue;
                        }
                    }
                } else {
                    declaration
                };

                let dependency_package = declaration
                    .get("package")
                    .and_then(Value::as_str)
                    .unwrap_or(alias);

                let requirement = declaration
                    .as_str()
                    .or_else(|| declaration.get("version")?.as_str());

                let resolved = Self::resolve(
                    locked_packages,
                    owner_name,
                    owner_version,
                    dependency_package,
                    requirement,
                );

                match resolved {
                    Ok((version, registry)) => {
                        scan.record(DependencyReference::new(
                            repository.clone(),
                            revision.to_owned(),
                            path.clone(),
                            import_name.clone(),
                            dependency_package.to_owned(),
                            version,
                            registry,
                            tokens.identifiers(&import_name),
                        ));
                    }

                    Err(error) => {
                        scan.fail(format!("{path}: {alias}: {error}",));
                    }
                }
            }
        }

        scan
    }

    fn collect_declarations<'a>(document: &'a Value, declarations: &mut Vec<(&'a str, &'a Value)>) {
        for key in Self::DEPENDENCY_TABLES {
            if let Some(table) = document.get(key).and_then(Value::as_table) {
                declarations.extend(table.iter().map(|(key, value)| (key.as_str(), value)));
            }
        }
    }

    fn ancestors(path: &str) -> Vec<&str> {
        let mut ancestors = Vec::new();
        let mut current = path;

        while let Some((parent, _)) = current.rsplit_once('/') {
            ancestors.push(parent);
            current = parent;
        }

        ancestors.push("");

        ancestors
    }

    fn join(directory: &str, filename: &str) -> String {
        if directory.is_empty() {
            filename.to_owned()
        } else {
            format!("{directory}/{filename}")
        }
    }

    fn nearest<'a>(
        documents: &'a BTreeMap<String, Value>,
        path: &str,
        filename: &str,
    ) -> Option<(&'a str, &'a Value)> {
        for directory in Self::ancestors(path) {
            let candidate = Self::join(directory, filename);

            if let Some((key, value)) = documents.get_key_value(&candidate) {
                return Some((key.as_str(), value));
            }
        }

        None
    }

    fn workspace<'a>(
        manifests: &'a BTreeMap<String, Value>,
        manifest_path: &str,
    ) -> Option<&'a Value> {
        for directory in Self::ancestors(manifest_path) {
            let candidate = Self::join(directory, "Cargo.toml");

            if let Some(workspace) = manifests
                .get(&candidate)
                .and_then(|manifest| manifest.get("workspace"))
            {
                return Some(workspace);
            }
        }

        None
    }

    fn resolve(
        packages: &[Value],
        owner_name: &str,
        owner_version: Option<&str>,
        dependency_name: &str,
        requirement: Option<&str>,
    ) -> Result<(Version, String), String> {
        let owners: Vec<_> = packages
            .iter()
            .filter(|package| {
                package.get("name").and_then(Value::as_str) == Some(owner_name)
                    && package.get("source").is_none()
                    && owner_version.is_none_or(|version| {
                        package.get("version").and_then(Value::as_str) == Some(version)
                    })
            })
            .collect();

        let [owner] = owners.as_slice() else {
            return Err("Ambiguous or missing locked owner".to_owned());
        };

        let dependencies = owner
            .get("dependencies")
            .and_then(Value::as_array)
            .ok_or("No resolved dependency edges for owner")?;

        let requirement = requirement
            .map(VersionReq::parse)
            .transpose()
            .map_err(|error| error.to_string())?;

        let candidates: Vec<_> = packages
            .iter()
            .filter(|package| package.get("name").and_then(Value::as_str) == Some(dependency_name))
            .filter(|package| {
                let Some(version) = package
                    .get("version")
                    .and_then(Value::as_str)
                    .and_then(|value| Version::parse(value).ok())
                else {
                    return false;
                };

                requirement
                    .as_ref()
                    .is_none_or(|requirement| requirement.matches(&version))
            })
            .filter(|package| {
                dependencies
                    .iter()
                    .filter_map(Value::as_str)
                    .any(|edge| Self::matches_edge(edge, package))
            })
            .collect();

        let [package] = candidates.as_slice() else {
            return Err(format!(
                "Expected one resolved dependency, found {}",
                candidates.len(),
            ));
        };

        let version = package
            .get("version")
            .and_then(Value::as_str)
            .ok_or("Missing locked dependency version")?;

        let source = package
            .get("source")
            .and_then(Value::as_str)
            .ok_or("Local dependency has no public registry source")?;

        let registry = source
            .strip_prefix("registry+")
            .or_else(|| source.strip_prefix("sparse+"))
            .ok_or("Dependency is not a registry package")?
            .trim_end_matches('/');

        if !matches!(
            registry,
            "https://github.com/rust-lang/crates.io-index" | "https://index.crates.io"
        ) {
            return Err(format!("No documentation provider for registry {registry}",));
        }

        Ok((
            Version::parse(version).map_err(|error| error.to_string())?,
            registry.to_owned(),
        ))
    }

    fn matches_edge(edge: &str, package: &Value) -> bool {
        let mut parts = edge.splitn(3, ' ');

        if parts.next() != package.get("name").and_then(Value::as_str) {
            return false;
        }

        if let Some(version) = parts.next()
            && package.get("version").and_then(Value::as_str) != Some(version)
        {
            return false;
        }

        if let Some(source) = parts.next() {
            let source = source.trim_start_matches('(').trim_end_matches(')');

            if package.get("source").and_then(Value::as_str) != Some(source) {
                return false;
            }
        }

        true
    }
}
