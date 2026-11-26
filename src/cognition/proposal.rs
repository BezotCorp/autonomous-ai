use crate::{
    reasoning::{epistemic_kind::EpistemicKind, node_id::NodeId},
    representation::{content::Content, provenance::Provenance},
};

/// Result proposed by a module, with the premises that justify it. The
/// engine decides whether and how it enters the reasoning graph.
#[derive(Debug)]
pub(crate) struct Proposal {
    content: Content,
    kind: EpistemicKind,
    premises: Vec<NodeId>,
    provenance: Option<Provenance>,
}

impl Proposal {
    pub(crate) fn new(content: Content, kind: EpistemicKind, premises: Vec<NodeId>) -> Self {
        Self {
            content,
            kind,
            premises,
            provenance: None,
        }
    }

    pub(crate) fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    pub(crate) fn kind(&self) -> EpistemicKind {
        self.kind
    }

    pub(crate) fn into_parts(self) -> (Content, EpistemicKind, Vec<NodeId>, Option<Provenance>) {
        (self.content, self.kind, self.premises, self.provenance)
    }
}
