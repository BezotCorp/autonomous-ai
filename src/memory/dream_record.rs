use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    memory::{
        dream_mode::DreamMode, dream_variant::DreamVariant, dream_verdict::DreamVerdict,
        episode_record::EpisodeRecord, outcome_digest::OutcomeDigest,
    },
    representation::rule::Rule,
};

/// Account of one dream: which experience it replayed, how, what it
/// found compared with the original, and its complete internal trace. Its
/// answers are simulated: they never count as observations of the world.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct DreamRecord {
    id: Uuid,
    original: Uuid,
    original_event: String,
    mode: DreamMode,
    variant: Option<DreamVariant>,
    reason: String,
    started_at: DateTime<Utc>,
    original_outcome: OutcomeDigest,
    dream_outcome: OutcomeDigest,
    verdict: DreamVerdict,
    findings: Vec<String>,
    hypotheses: Vec<Rule>,
    episode: Option<EpisodeRecord>,
}

impl DreamRecord {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        original: &EpisodeRecord,
        mode: DreamMode,
        variant: Option<DreamVariant>,
        reason: String,
        started_at: DateTime<Utc>,
        dream_outcome: OutcomeDigest,
        verdict: DreamVerdict,
        findings: Vec<String>,
        hypotheses: Vec<Rule>,
        episode: Option<EpisodeRecord>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            original: original.id(),
            original_event: original.event_id().to_owned(),
            mode,
            variant,
            reason,
            started_at,
            original_outcome: OutcomeDigest::of(original),
            dream_outcome,
            verdict,
            findings,
            hypotheses,
            episode,
        }
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn original(&self) -> Uuid {
        self.original
    }

    pub(crate) fn mode(&self) -> DreamMode {
        self.mode
    }

    pub(crate) fn variant(&self) -> Option<&DreamVariant> {
        self.variant.as_ref()
    }

    pub(crate) fn verdict(&self) -> DreamVerdict {
        self.verdict
    }

    pub(crate) fn findings(&self) -> &[String] {
        &self.findings
    }

    pub(crate) fn hypotheses(&self) -> &[Rule] {
        &self.hypotheses
    }

    pub(crate) fn original_outcome(&self) -> &OutcomeDigest {
        &self.original_outcome
    }

    pub(crate) fn dream_outcome(&self) -> &OutcomeDigest {
        &self.dream_outcome
    }

    pub(crate) fn episode(&self) -> Option<&EpisodeRecord> {
        self.episode.as_ref()
    }

    /// Drops the detailed trace, keeping the comparison.
    pub(crate) fn forget_trace(&mut self) {
        self.episode = None;
    }

    pub(crate) fn started_at(&self) -> DateTime<Utc> {
        self.started_at
    }
}
