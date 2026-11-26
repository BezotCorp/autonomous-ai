use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::memory::{answer_status::AnswerStatus, episode_record::EpisodeRecord};

/// What an episode achieved and what it cost, in a form comparable
/// between an original experience and its replays.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct OutcomeDigest {
    answers: BTreeMap<String, (String, BTreeSet<String>)>,
    invocations: usize,
    modules: BTreeMap<String, usize>,
    useless_invocations: usize,
    consumption: BTreeMap<String, f64>,
    cost: f64,
    strategies: Vec<String>,
}

impl OutcomeDigest {
    pub(crate) fn of(episode: &EpisodeRecord) -> Self {
        let mut digest = Self::default();

        for answer in episode.answers() {
            let status = match answer.status() {
                AnswerStatus::Proven => "proven",
                AnswerStatus::Refuted => "refuted",
                AnswerStatus::Contradictory => "contradictory",
                AnswerStatus::Undetermined => "undetermined",
            };

            digest.answers.insert(
                answer.question().to_string(),
                (
                    status.to_owned(),
                    answer.answers().iter().map(ToString::to_string).collect(),
                ),
            );
        }

        for invocation in episode.invocations() {
            digest.invocations += 1;
            digest.cost += invocation.actual_cost();
            *digest
                .modules
                .entry(invocation.module().to_owned())
                .or_default() += 1;

            if !invocation.outcome().is_success(true) {
                digest.useless_invocations += 1;
            }
        }

        digest.consumption = episode.usage().consumption().clone();
        digest.strategies = episode
            .plans()
            .iter()
            .map(|record| record.plan().strategy().to_owned())
            .collect();
        digest
    }

    /// Number of questions with a definite answer (proven or refuted).
    pub(crate) fn definite_answers(&self) -> usize {
        self.answers
            .values()
            .filter(|(status, _)| status == "proven" || status == "refuted")
            .count()
    }

    pub(crate) fn same_answers(&self, other: &Self) -> bool {
        self.answers == other.answers
    }

    pub(crate) fn invocations(&self) -> usize {
        self.invocations
    }

    pub(crate) fn uses(&self, module: &str) -> bool {
        self.modules.contains_key(module)
    }

    pub(crate) fn modules(&self) -> impl Iterator<Item = &String> {
        self.modules.keys()
    }

    pub(crate) fn total_consumption(&self) -> f64 {
        // Adding zero turns a negative zero into zero.
        self.consumption.values().sum::<f64>() + 0.0
    }
}
