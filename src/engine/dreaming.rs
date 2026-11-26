use std::{collections::BTreeSet, time::Instant};

use chrono::Duration;

use crate::{
    acquisition::access_mode::AccessMode,
    engine::{disposition::Disposition, intelligence::Intelligence},
    memory::{
        activity::Activity, dream_mode::DreamMode, dream_record::DreamRecord,
        dream_scene::DreamScene, dream_variant::DreamVariant, dream_verdict::DreamVerdict,
        episode_configuration::EpisodeConfiguration, episode_record::EpisodeRecord,
        outcome_digest::OutcomeDigest, persistent_memory::PersistentMemory,
    },
};

/// Replay of a real experience inside a dream, as one of its scenes. The
/// ordinary engine runs on the dream's isolated memory, in replay mode:
/// modules read only what was recorded when the experience happened,
/// nothing reaches the environment, and what was not recorded stays
/// missing. A variant may change the sequence of decisions, remove a
/// capability or an influence, or impose a scarcity. The scene is compared
/// with the original.
pub(crate) struct Dreaming;

impl Dreaming {
    /// Relative difference below which two outcomes are equivalent.
    const TOLERANCE: f64 = 0.1;

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn replay(
        intelligence: &Intelligence,
        sandbox: &mut PersistentMemory,
        original: &EpisodeRecord,
        mode: DreamMode,
        variant: Option<DreamVariant>,
        seed: u64,
        transition: String,
        disposition: &mut dyn Disposition,
    ) -> DreamScene {
        let mechanisms = vec![match mode {
            DreamMode::Historical => "replay".to_owned(),
            DreamMode::Reinterpretation => "reinterpretation".to_owned(),
        }]
        .into_iter()
        .chain(variant.as_ref().map(|variant| format!("variant:{variant}")))
        .collect::<Vec<_>>();

        let Some(event) = original.event() else {
            return DreamScene::failed(
                mechanisms,
                Some(original.id()),
                "the event of the experience was not recorded".to_owned(),
            );
        };

        let since = original.started_at();
        let as_of = since
            + Duration::microseconds(i64::try_from(original.duration_micros()).unwrap_or(i64::MAX));
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
            Some(DreamVariant::WithoutInfluence { influence }) => {
                configuration = configuration.excluding_influences(BTreeSet::from([*influence]));
            }
            Some(DreamVariant::Reorder { .. }) | None => {}
        }

        // Replays are free of influences, so that what they test is not
        // mixed with them; a replay testing one influence keeps the others.
        if !matches!(variant, Some(DreamVariant::WithoutInfluence { .. })) {
            configuration = configuration.without_influences();
        }

        let clock = Instant::now();
        let report = intelligence.reason_with(event, sandbox, configuration, false, disposition);
        let episode = report.episode().clone();
        let before = OutcomeDigest::of(original);
        let after = OutcomeDigest::of(&episode);
        let verdict = Self::verdict(&before, &after);
        let findings = Self::findings(&before, &after, verdict, variant.as_ref());
        let consequences = episode
            .answers()
            .iter()
            .flat_map(|answer| answer.answers().iter().cloned())
            .collect();

        DreamScene::new(
            mechanisms,
            transition,
            Vec::new(),
            Vec::new(),
            original.questions().to_vec(),
            episode,
            consequences,
            clock.elapsed().as_secs_f64(),
        )
        .replaying(
            original.id(),
            mode,
            variant,
            (before, after, verdict),
            findings,
        )
    }

    /// Records a dream in the real memory. What it shows is kept as
    /// simulated experience, apart from real learning; only replays testing
    /// an influence count, as demonstrations of what that influence
    /// changes.
    pub(crate) fn integrate(memory: &mut PersistentMemory, dream: DreamRecord) {
        for scene in dream.scenes() {
            if let Some(episode) = scene.episode() {
                memory.parts_mut().1.learn_simulated(episode);
            }

            if let (Some(DreamVariant::WithoutInfluence { influence }), Some((before, after, _))) =
                (scene.variant(), scene.comparison())
            {
                memory.influences_mut().demonstrate(
                    *influence,
                    dream.id(),
                    !after.same_answers(before),
                );
            }
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
        let lower = |after: f64, before: f64| after < before * (1.0 - Self::TOLERANCE);
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
