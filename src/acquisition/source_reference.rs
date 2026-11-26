use std::collections::BTreeSet;

/// Use of a dependency in one source file: the resolved
/// package version and registry, and the symbols referenced through it.
#[derive(Clone, Debug)]
pub(crate) struct SourceReference {
    file: String,
    package: String,
    version: semver::Version,
    registry: String,
    symbols: BTreeSet<String>,
}

impl SourceReference {
    pub(crate) fn new(
        file: String,
        package: String,
        version: semver::Version,
        registry: String,
        symbols: BTreeSet<String>,
    ) -> Self {
        Self {
            file,
            package,
            version,
            registry,
            symbols,
        }
    }

    pub(crate) fn file(&self) -> &str {
        &self.file
    }

    pub(crate) fn package(&self) -> &str {
        &self.package
    }

    pub(crate) fn version(&self) -> &semver::Version {
        &self.version
    }

    pub(crate) fn registry(&self) -> &str {
        &self.registry
    }

    pub(crate) fn symbols(&self) -> &BTreeSet<String> {
        &self.symbols
    }
}
