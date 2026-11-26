use std::collections::BTreeSet;

use chrono::{Duration, Utc};

use crate::{
    acquisition::access_mode::AccessMode,
    engine::intelligence::Intelligence,
    memory::{
        activity::Activity, dream_mode::DreamMode, dream_record::DreamRecord,
        dream_variant::DreamVariant, dream_verdict::DreamVerdict,
        episode_configuration::EpisodeConfiguration, episode_record::EpisodeRecord,
        outcome_digest::OutcomeDigest, persistent_memory::PersistentMemory,
    },
};

/// Internal exploration of past experiences. A dream runs the ordinary
/// engine on an isolated copy of the memory, in replay mode: modules read
/// only what was recorded when the experience happened, nothing reaches
/// the environment, and what was not recorded stays missing.
///
/// - historical replay: the memory as it stood then;
/// - reinterpretation: the memory as it stands now, on the data of then.
///
/// A variant may change the sequence of decisions, remove a capability or
/// impose a scarcity. The dream is then compared with the original.
pub(crate) struct Dreaming;

impl Dreaming {
    /// Relative cost difference below which two outcomes are equivalent.
    const COST_TOLERANCE: f64 = 0.1;

    pub(crate) fn dream(
        intelligence: &Intelligence,
        memory: &PersistentMemory,
        original: &EpisodeRecord,
        mode: DreamMode,
        variant: Option<DreamVariant>,
        reason: String,
        seed: u64,
    ) -> DreamRecord {
        let started_at = Utc::now();
        let fail = |finding: String| {
            DreamRecord::new(
                original,
                mode,
                variant.clone(),
                reason.clone(),
                started_at,
                OutcomeDigest::default(),
                DreamVerdict::Failed,
                vec![finding],
                Vec::new(),
                None,
            )
        };

        let Some(event) = original.event() else {
            return fail("the event of the experience was not recorded".to_owned());
        };

        let since = original.started_at();
        let as_of = since
            + Duration::microseconds(i64::try_from(original.duration_micros()).unwrap_or(i64::MAX));

        let mut sandbox = match mode {
            DreamMode::Historical => memory.as_of(since),
            DreamMode::Reinterpretation => memory.clone(),
        };

        let mut configuration = EpisodeConfiguration::new(seed, true, true)
            .with_activity(Activity::Dream)
            .with_access(AccessMode::Replay { as_of, since });

        match &variant {
            Some(DreamVariant::Ablation { module }) => {
                configuration = configuration.without_modules(BTreeSet::from([module.clone()]));
            }
            Some(DreamVariant::Scarcity { resource, units }) => {
                configuration =
                    configuration.with_resource_budgets([(resource.clone(), *units as f64)].into());
            }
            Some(DreamVariant::Reorder { .. }) | None => {}
        }

        let report = intelligence.reason(event, &mut sandbox, configuration, false);
        let episode = report.episode().clone();
        let before = OutcomeDigest::of(original);
        let after = OutcomeDigest::of(&episode);
        let verdict = Self::verdict(&before, &after);
        let mut findings = Self::findings(&before, &after, verdict, variant.as_ref());

        for invocation in episode.invocations() {
            if let Some(error) = invocation.error()
                && error.contains("left missing")
            {
                findings.push(format!(
                    "{}: data not recorded at the time, left missing",
                    invocation.module()
                ));
            }
        }

        let hypotheses = report
            .knowledge_changes()
            .hypotheses_proposed()
            .iter()
            .filter(|rule| memory.knowledge().hypothesis(rule).is_none())
            .cloned()
            .collect();

        DreamRecord::new(
            original,
            mode,
            variant,
            reason,
            started_at,
            after,
            verdict,
            findings,
            hypotheses,
            Some(episode),
        )
    }

    /// Brings what a dream taught into the real memory, kept apart from
    /// what was learned in reality: simulated routing statistics and
    /// strategies, and conjectures marked as dreamt.
    pub(crate) fn integrate(memory: &mut PersistentMemory, dream: DreamRecord) {
        let (knowledge, routing, _) = memory.parts_mut();

        if let Some(episode) = dream.episode() {
            routing.learn_simulated(episode);
        }

        for rule in dream.hypotheses() {
            knowledge.propose_dreamt_hypothesis(rule.clone(), dream.id());
        }

        memory.dreams_mut().record(dream);
    }

    fn verdict(before: &OutcomeDigest, after: &OutcomeDigest) -> DreamVerdict {
        if after.definite_answers() > before.definite_answers() {
            return DreamVerdict::MoreAnswers;
        }

        if !after.same_answers(before) {
            return DreamVerdict::Worse;
        }

        // Durations are not compared: a dream never waits for the network.
        let lower = |after: f64, before: f64| after < before * (1.0 - Self::COST_TOLERANCE);
        let consumption = (after.total_consumption(), before.total_consumption());
        let invocations = (after.invocations() as f64, before.invocations() as f64);

        if lower(consumption.0, consumption.1)
            || (consumption.0 <= consumption.1 && lower(invocations.0, invocations.1))
        {
            DreamVerdict::Cheaper
        } else {
            DreamVerdict::Equivalent
        }
    }

    fn findings(
        before: &OutcomeDigest,
        after: &OutcomeDigest,
        verdict: DreamVerdict,
        variant: Option<&DreamVariant>,
    ) -> Vec<String> {
        let mut findings = vec![format!(
            "invocations {} -> {}, consumption {} -> {}",
            before.invocations(),
            after.invocations(),
            before.total_consumption(),
            after.total_consumption()
        )];

        if matches!(verdict, DreamVerdict::Cheaper | DreamVerdict::Equivalent) {
            for module in before.modules().filter(|module| !after.uses(module)) {
                findings.push(format!("unnecessary: {module} (same answers without it)"));
            }
        }

        match (variant, verdict) {
            (Some(DreamVariant::Ablation { module }), DreamVerdict::Worse) => {
                findings.push(format!("necessary: {module} (answers lost without it)"));
            }
            (_, DreamVerdict::MoreAnswers) => {
                findings.push("more answers in simulation: to verify in reality".to_owned());
            }
            _ => {}
        }

        findings
    }
}
