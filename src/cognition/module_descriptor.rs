use serde::Serialize;

use crate::{reasoning::epistemic_kind::EpistemicKind, representation::content_kind::ContentKind};

/// Self-description of a cognitive module: what it consumes, which needs
/// it is able to answer, what kind of results it produces and whether it
/// reaches external sources. The engine relies on it to know which modules
/// *could* act; deciding which one *should* act is learned separately.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ModuleDescriptor {
    name: String,
    purpose: String,
    consumes: Vec<ContentKind>,
    supplies: Vec<String>,
    produces: Vec<EpistemicKind>,
    external: bool,
}

impl ModuleDescriptor {
    pub(crate) fn new(name: &str, purpose: &str) -> Self {
        Self {
            name: name.to_owned(),
            purpose: purpose.to_owned(),
            consumes: Vec::new(),
            supplies: Vec::new(),
            produces: Vec::new(),
            external: false,
        }
    }

    pub(crate) fn consuming(mut self, kinds: &[ContentKind]) -> Self {
        self.consumes.extend_from_slice(kinds);
        self
    }

    /// Restricts the needs this module answers to the given signatures
    /// (`predicate/arity`). Without restriction, every need is relevant.
    pub(crate) fn supplying(mut self, signatures: &[&str]) -> Self {
        self.supplies
            .extend(signatures.iter().map(|signature| (*signature).to_owned()));
        self
    }

    pub(crate) fn producing(mut self, kinds: &[EpistemicKind]) -> Self {
        self.produces.extend_from_slice(kinds);
        self
    }

    pub(crate) fn external(mut self) -> Self {
        self.external = true;
        self
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn consumes(&self, kind: ContentKind) -> bool {
        self.consumes.contains(&kind)
    }

    pub(crate) fn supplies(&self, signature: &str) -> bool {
        self.supplies.is_empty() || self.supplies.iter().any(|supplied| supplied == signature)
    }

    pub(crate) fn produces(&self, kind: EpistemicKind) -> bool {
        self.produces.contains(&kind)
    }

    pub(crate) fn is_external(&self) -> bool {
        self.external
    }
}
