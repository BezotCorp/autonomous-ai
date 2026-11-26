use serde::Serialize;

use crate::reasoning::node_id::NodeId;

/// Two valid results asserting a statement and its negation.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct Contradiction {
    positive: NodeId,
    negative: NodeId,
    step: usize,
}

impl Contradiction {
    pub(crate) fn new(positive: NodeId, negative: NodeId, step: usize) -> Self {
        Self {
            positive,
            negative,
            step,
        }
    }

    pub(crate) fn sides(&self) -> [NodeId; 2] {
        [self.positive, self.negative]
    }
}
