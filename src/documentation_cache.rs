use std::collections::BTreeMap;

use crate::{
    documentation_provider::DocumentationProvider, package_documentation::PackageDocumentation,
};

#[derive(Default)]
pub(crate) struct DocumentationCache {
    packages: BTreeMap<(String, semver::Version), PackageDocumentation>,
}

impl DocumentationCache {
    pub(crate) fn acquire(
        &mut self,
        package: &str,
        version: &semver::Version,
    ) -> &PackageDocumentation {
        self.packages
            .entry((package.to_owned(), version.clone()))
            .or_insert_with(|| DocumentationProvider::acquire(package, version))
    }
}
