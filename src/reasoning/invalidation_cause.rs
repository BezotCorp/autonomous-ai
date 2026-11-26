use serde::Serialize;

use crate::reasoning::node_id::NodeId;

/// Why a result stopped being considered valid.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InvalidationCause {
    /// Withdrawn by belief revision after a contradiction.
    Retracted { contradiction: usize },
    /// One of its premises was invalidated.
    DependsOn { premise: NodeId },
    /// Part of a contradiction that could not be attributed to a defeasible cause.
    Contested { contradiction: usize },
}
