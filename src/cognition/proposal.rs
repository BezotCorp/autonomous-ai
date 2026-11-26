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
    staleness: f64,
}

impl Proposal {
    pub(crate) fn new(content: Content, kind: EpistemicKind, premises: Vec<NodeId>) -> Self {
        Self {
            content,
            kind,
            premises,
            provenance: None,
            staleness: 0.0,
        }
    }

    pub(crate) fn with_provenance(mut self, provenance: Provenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    /// Probability that the proposed content no longer holds now, for
    /// content remembered from an earlier observation.
    pub(crate) fn with_staleness(mut self, staleness: f64) -> Self {
        self.staleness = staleness;
        self
    }

    pub(crate) fn staleness(&self) -> f64 {
        self.staleness
    }

    pub(crate) fn provenance(&self) -> Option<&Provenance> {
        self.provenance.as_ref()
    }

    pub(crate) fn content(&self) -> &Content {
        &self.content
    }

    pub(crate) fn kind(&self) -> EpistemicKind {
        self.kind
    }

    pub(crate) fn into_parts(self) -> (Content, EpistemicKind, Vec<NodeId>, Option<Provenance>) {
        (self.content, self.kind, self.premises, self.provenance)
    }
}
