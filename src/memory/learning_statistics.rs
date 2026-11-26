use std::collections::BTreeMap;

use serde::Serialize;

use crate::memory::{
    answer_status::AnswerStatus, configuration_statistics::ConfigurationStatistics,
    hypothesis_status::HypothesisStatus, module_statistics::ModuleStatistics,
    persistent_memory::PersistentMemory, standing::Standing,
};

/// Statistics recomputed from the memories, never stored: frequencies of
/// use, outcomes, costs, answers, revisions and the state of knowledge.
#[derive(Debug, Default, Serialize)]
pub(crate) struct LearningStatistics {
    episodes: usize,
    mean_episode_seconds: f64,
    terminations: BTreeMap<String, usize>,
    configurations: BTreeMap<String, ConfigurationStatistics>,
    answers: BTreeMap<String, usize>,
    contradictions: usize,
    root_impasses: BTreeMap<String, usize>,
    dependent_impasses: usize,
    hypotheses_proposed: usize,
    hypotheses_refuted: usize,
    modules: BTreeMap<String, ModuleStatistics>,
    knowledge_observed: usize,
    knowledge_derived: usize,
    knowledge_obsolete: usize,
    hypotheses_open: usize,
    hypotheses_marked_refuted: usize,
    hypothesis_mean_support: f64,
}

impl LearningStatistics {
    pub(crate) fn compute(memory: &PersistentMemory) -> Self {
        let mut statistics = Self::default();
        let mut total_micros = 0_u64;

        for episode in memory.experiences().episodes() {
            statistics.episodes += 1;
            total_micros = total_micros.saturating_add(episode.duration_micros());

            statistics
                .configurations
                .entry(ConfigurationStatistics::label(episode))
                .or_default()
                .observe(episode);

            *statistics
                .terminations
                .entry(format!("{:?}", episode.termination()))
                .or_default() += 1;

            for answer in episode.answers() {
                let status = match answer.status() {
                    AnswerStatus::Proven => "proven",
                    AnswerStatus::Refuted => "refuted",
                    AnswerStatus::Contradictory => "contradictory",
                    AnswerStatus::Undetermined => "undetermined",
                };

                *statistics.answers.entry(status.to_owned()).or_default() += 1;
            }

            statistics.contradictions += episode.revisions().len();

            for impasse in episode.impasses() {
                if impasse.kind().is_root() {
                    *statistics
                        .root_impasses
                        .entry(format!(
                            "{} {}",
                            impasse.kind().label(),
                            impasse.need().signature()
                        ))
                        .or_default() += 1;
                } else {
                    statistics.dependent_impasses += 1;
                }
            }
            statistics.hypotheses_proposed += episode.hypotheses_proposed().len();
            statistics.hypotheses_refuted += episode.hypotheses_refuted().len();

            for invocation in episode.invocations() {
                statistics
                    .modules
                    .entry(invocation.module().to_owned())
                    .or_default()
                    .observe(invocation.outcome(), invocation.duration_micros());
            }
        }

        if statistics.episodes > 0 {
            statistics.mean_episode_seconds =
                total_micros as f64 / 1e6 / statistics.episodes as f64;
        }

        for entry in memory.knowledge().entries() {
            match entry.standing() {
                Standing::Observed => statistics.knowledge_observed += 1,
                Standing::Derived => statistics.knowledge_derived += 1,
                Standing::Obsolete { .. } => statistics.knowledge_obsolete += 1,
            }
        }

        let mut support = 0;

        for hypothesis in memory.knowledge().hypotheses() {
            support += hypothesis.support();

            match hypothesis.status() {
                HypothesisStatus::Open => statistics.hypotheses_open += 1,
                HypothesisStatus::Refuted => statistics.hypotheses_marked_refuted += 1,
            }
        }

        let hypotheses = statistics.hypotheses_open + statistics.hypotheses_marked_refuted;

        if hypotheses > 0 {
            statistics.hypothesis_mean_support = support as f64 / hypotheses as f64;
        }

        statistics
    }
}
