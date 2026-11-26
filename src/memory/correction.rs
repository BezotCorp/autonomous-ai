use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    memory::invocation_outcome::InvocationOutcome,
    representation::{rule::Rule, statement::Statement},
};

/// Change of the memory proposed by the examination of the cognition.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Correction {
    /// A conjecture contradicted by what is known.
    RefuteHypothesis { rule: Rule },
    /// A refutation whose grounds disappeared.
    ReopenHypothesis { rule: Rule },
    /// A belief that should no longer be held.
    ObsoleteFact { statement: Statement },
    /// An influence whose real consequences do not justify it any more.
    RetireInfluence { influence: Uuid },
    /// The credit given to a past invocation, re-examined.
    ReassessInvocation {
        episode: Uuid,
        step: usize,
        module: String,
        outcome: InvocationOutcome,
    },
}

impl Correction {
    /// What the correction is about, to avoid correcting it twice.
    pub(crate) fn subject(&self) -> String {
        match self {
            Self::RefuteHypothesis { rule } | Self::ReopenHypothesis { rule } => rule.to_string(),
            Self::ObsoleteFact { statement } => statement.to_string(),
            Self::ReassessInvocation { episode, step, .. } => format!("{episode}#{step}"),
            Self::RetireInfluence { influence } => influence.to_string(),
        }
    }
}

impl fmt::Display for Correction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RefuteHypothesis { rule } => write!(formatter, "refute {rule}"),
            Self::ReopenHypothesis { rule } => write!(formatter, "reopen {rule}"),
            Self::ObsoleteFact { statement } => write!(formatter, "withdraw {statement}"),
            Self::RetireInfluence { influence } => {
                write!(formatter, "retire influence {influence}")
            }
            Self::ReassessInvocation {
                episode,
                step,
                module,
                outcome,
            } => write!(
                formatter,
                "reassess {module} at {episode}#{step} as {outcome:?}"
            ),
        }
    }
}
