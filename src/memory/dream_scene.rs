use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    memory::{
        dream_mode::DreamMode, dream_variant::DreamVariant, dream_verdict::DreamVerdict,
        episode_record::EpisodeRecord, imagined_capability_spec::ImaginedCapabilitySpec,
        imagined_element::ImaginedElement, outcome_digest::OutcomeDigest,
    },
    representation::statement::Statement,
};

/// One scene of a dream: the mechanisms that shaped it, how it followed
/// from the previous one, its setting (each element with its provenance),
/// the capabilities imagined in it, the questions it raised, the needs that
/// emerged (what nothing could provide), the consequences imagined, the
/// strategies explored and, for a replay, its comparison with reality.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct DreamScene {
    mechanisms: Vec<String>,
    transition: String,
    origin: Option<Uuid>,
    mode: Option<DreamMode>,
    variant: Option<DreamVariant>,
    elements: Vec<ImaginedElement>,
    capabilities: Vec<ImaginedCapabilitySpec>,
    questions: Vec<Statement>,
    emerged_needs: Vec<Statement>,
    consequences: Vec<Statement>,
    strategies: Vec<String>,
    comparison: Option<(OutcomeDigest, OutcomeDigest, DreamVerdict)>,
    findings: Vec<String>,
    seconds: f64,
    reasoning_steps: usize,
    episode: Option<EpisodeRecord>,
}

impl DreamScene {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        mechanisms: Vec<String>,
        transition: String,
        elements: Vec<ImaginedElement>,
        capabilities: Vec<ImaginedCapabilitySpec>,
        questions: Vec<Statement>,
        episode: EpisodeRecord,
        consequences: Vec<Statement>,
        seconds: f64,
    ) -> Self {
        let emerged_needs = episode
            .impasses()
            .iter()
            .filter(|impasse| impasse.kind().is_root())
            .map(|impasse| impasse.need().clone())
            .collect();
        let strategies = episode
            .plans()
            .iter()
            .map(|record| record.plan().strategy().to_owned())
            .collect();
        let findings = episode
            .invocations()
            .iter()
            .filter_map(|invocation| {
                invocation
                    .error()
                    .filter(|error| error.contains("left missing"))
                    .map(|_| {
                        format!(
                            "{}: data not recorded at the time, left missing",
                            invocation.module()
                        )
                    })
            })
            .collect();

        Self {
            mechanisms,
            transition,
            elements,
            capabilities,
            questions,
            emerged_needs,
            consequences,
            strategies,
            findings,
            seconds,
            reasoning_steps: episode.invocations().len(),
            episode: Some(episode),
            ..Self::default()
        }
    }

    /// Marks the scene as the replay of a real experience, compared with it.
    pub(crate) fn replaying(
        mut self,
        origin: Uuid,
        mode: DreamMode,
        variant: Option<DreamVariant>,
        comparison: (OutcomeDigest, OutcomeDigest, DreamVerdict),
        findings: Vec<String>,
    ) -> Self {
        self.origin = Some(origin);
        self.mode = Some(mode);
        self.variant = variant;
        self.comparison = Some(comparison);
        self.findings.extend(findings);
        self
    }

    /// A scene that could not take place.
    pub(crate) fn failed(mechanisms: Vec<String>, origin: Option<Uuid>, finding: String) -> Self {
        Self {
            mechanisms,
            transition: "interrupted".to_owned(),
            origin,
            findings: vec![finding],
            comparison: None,
            ..Self::default()
        }
    }

    pub(crate) fn mechanisms(&self) -> &[String] {
        &self.mechanisms
    }

    pub(crate) fn transition(&self) -> &str {
        &self.transition
    }

    pub(crate) fn origin(&self) -> Option<Uuid> {
        self.origin
    }

    pub(crate) fn mode(&self) -> Option<DreamMode> {
        self.mode
    }

    pub(crate) fn variant(&self) -> Option<&DreamVariant> {
        self.variant.as_ref()
    }

    pub(crate) fn verdict(&self) -> Option<DreamVerdict> {
        self.comparison.as_ref().map(|(_, _, verdict)| *verdict)
    }

    pub(crate) fn comparison(&self) -> Option<&(OutcomeDigest, OutcomeDigest, DreamVerdict)> {
        self.comparison.as_ref()
    }

    pub(crate) fn elements(&self) -> &[ImaginedElement] {
        &self.elements
    }

    pub(crate) fn capabilities(&self) -> &[ImaginedCapabilitySpec] {
        &self.capabilities
    }

    pub(crate) fn questions(&self) -> &[Statement] {
        &self.questions
    }

    pub(crate) fn emerged_needs(&self) -> &[Statement] {
        &self.emerged_needs
    }

    pub(crate) fn consequences(&self) -> &[Statement] {
        &self.consequences
    }

    pub(crate) fn strategies(&self) -> &[String] {
        &self.strategies
    }

    pub(crate) fn findings(&self) -> &[String] {
        &self.findings
    }

    pub(crate) fn seconds(&self) -> f64 {
        self.seconds
    }

    pub(crate) fn episode(&self) -> Option<&EpisodeRecord> {
        self.episode.as_ref()
    }

    pub(crate) fn forget_trace(&mut self) {
        self.episode = None;
    }
}
