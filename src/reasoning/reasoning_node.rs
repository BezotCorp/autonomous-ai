use serde::Serialize;

use crate::{
    reasoning::{
        epistemic_kind::EpistemicKind, node_id::NodeId, producer::Producer, validity::Validity,
    },
    representation::{content::Content, provenance::Provenance},
};

/// One intermediate result of a reasoning, with its full justification.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ReasoningNode {
    id: NodeId,
    content: Content,
    kind: EpistemicKind,
    producer: Producer,
    premises: Vec<NodeId>,
    provenance: Option<Provenance>,
    validity: Validity,
    step: usize,
    revision: u64,
}

impl ReasoningNode {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: NodeId,
        content: Content,
        kind: EpistemicKind,
        producer: Producer,
        premises: Vec<NodeId>,
        provenance: Option<Provenance>,
        step: usize,
        revision: u64,
    ) -> Self {
        Self {
            id,
            content,
            kind,
            producer,
            premises,
            provenance,
            validity: Validity::Valid,
            step,
            revision,
        }
    }

    pub(crate) fn id(&self) -> NodeId {
        self.id
    }

    pub(crate) fn content(&self) -> &Content {
        &self.content
    }

    pub(crate) fn kind(&self) -> EpistemicKind {
        self.kind
    }

    pub(crate) fn producer(&self) -> &Producer {
        &self.producer
    }

    pub(crate) fn premises(&self) -> &[NodeId] {
        &self.premises
    }

    pub(crate) fn provenance(&self) -> Option<&Provenance> {
        self.provenance.as_ref()
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.validity.is_valid()
    }

    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn invalidate(&mut self, validity: Validity, revision: u64) {
        self.validity = validity;
        self.revision = revision;
    }
}
