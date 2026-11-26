use std::collections::BTreeMap;

use serde::Serialize;

use crate::{
    consumables::{resource_state::ResourceState, resource_usage::ResourceUsage},
    memory::{
        answer_status::AnswerStatus, configuration_statistics::ConfigurationStatistics,
        dream_statistics::DreamStatistics, hypothesis_status::HypothesisStatus,
        module_statistics::ModuleStatistics, persistent_memory::PersistentMemory,
        standing::Standing,
    },
    planning::strategy_statistic::StrategyStatistic,
};

/// Statistics recomputed from the memories, never stored: frequencies of
/// use, outcomes, costs, answers, revisions, consumption of external
/// resources, strategies and the state of knowledge.
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
    usage: ResourceUsage,
    resources: BTreeMap<String, ResourceState>,
    strategies: BTreeMap<String, BTreeMap<String, StrategyStatistic>>,
    observed_scopes: usize,
    scope_verifications: u64,
    scope_changes: u64,
    acquisition_cache_entries: usize,
    acquisition_cache_bytes: usize,
    dreams: DreamStatistics,
    pending_experiments: usize,
    corrections_applied: BTreeMap<String, usize>,
    corrections_reverted: usize,
    activities: BTreeMap<String, (usize, usize, f64)>,
    influences_active: usize,
    influences_retired: usize,
    influence_exposures: u64,
    decisions_changed_by_influences: usize,
    influence_attributions: BTreeMap<String, usize>,
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
            statistics.usage.add(episode.usage());

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

        for (_, scope) in memory.knowledge().scopes() {
            statistics.observed_scopes += 1;
            statistics.scope_verifications += scope.verifications();
            statistics.scope_changes += scope.changes();
        }

        statistics.resources = memory.resources().clone();
        statistics.strategies = memory.routing().strategies().clone();
        statistics.acquisition_cache_entries = memory.acquisitions().len();
        statistics.acquisition_cache_bytes = memory.acquisitions().bytes();

        for dream in memory.dreams().dreams() {
            statistics.dreams.observe(dream);
        }

        statistics.pending_experiments = memory.dreams().pending().len();

        for record in memory.corrections().records() {
            let kind = format!("{:?}", record.correction())
                .split([' ', '{'])
                .next()
                .unwrap_or_default()
                .to_owned();

            *statistics.corrections_applied.entry(kind).or_default() += 1;

            if !record.is_active() {
                statistics.corrections_reverted += 1;
            }
        }

        for record in memory.activities().records() {
            let entry = statistics
                .activities
                .entry(record.focus().to_owned())
                .or_default();

            entry.0 += 1;
            entry.2 += record.seconds();

            if record.worthwhile() {
                entry.1 += 1;
            }
        }

        let evidence = memory.influences().evidence();

        for influence in memory.influences().influences() {
            if influence.is_active() {
                statistics.influences_active += 1;
            } else {
                statistics.influences_retired += 1;
            }

            let evidence = evidence.get(&influence.id()).copied().unwrap_or_default();

            statistics.influence_exposures += evidence.exposures();
            *statistics
                .influence_attributions
                .entry(evidence.attribution().to_owned())
                .or_default() += 1;
        }

        statistics.decisions_changed_by_influences = memory
            .real_episodes()
            .flat_map(|episode| episode.invocations())
            .filter(|invocation| !invocation.selection().influences().is_empty())
            .count();

        let hypotheses = statistics.hypotheses_open + statistics.hypotheses_marked_refuted;

        if hypotheses > 0 {
            statistics.hypothesis_mean_support = support as f64 / hypotheses as f64;
        }

        statistics
    }
}
