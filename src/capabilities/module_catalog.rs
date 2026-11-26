use crate::{
    capabilities::{
        cargo_observation::CargoObservation, documentation_observation::DocumentationObservation,
        evidence_review::EvidenceReview, hypothesis_formation::HypothesisFormation,
        knowledge_retrieval::KnowledgeRetrieval, learning_review::LearningReview,
        linguistic_analysis::LinguisticAnalysis, logical_inference::LogicalInference,
        mathematical_reasoning::MathematicalReasoning, relational_analysis::RelationalAnalysis,
        release_observation::ReleaseObservation, repository_discovery::RepositoryDiscovery,
        source_reference_observation::SourceReferenceObservation,
        strategic_planning::StrategicPlanning,
    },
    cognition::{cognitive_module::CognitiveModule, module_registry::ModuleRegistry},
};

/// Assembles the registry of available capabilities.
pub(crate) struct ModuleCatalog;

impl ModuleCatalog {
    /// Distinct examples required before conjecturing a generalisation.
    const INDUCTION_MINIMUM_SUPPORT: usize = 3;

    /// Every capability, or only those that do not reach external sources.
    pub(crate) fn standard(offline: bool) -> ModuleRegistry {
        let mut registry = ModuleRegistry::default();

        Self::add(&mut registry, offline, LogicalInference::new());
        Self::add(&mut registry, offline, MathematicalReasoning::new());
        Self::add(&mut registry, offline, RelationalAnalysis::new());
        Self::add(
            &mut registry,
            offline,
            HypothesisFormation::new(Self::INDUCTION_MINIMUM_SUPPORT),
        );
        Self::add(&mut registry, offline, KnowledgeRetrieval::new());
        Self::add(&mut registry, offline, StrategicPlanning::new());
        Self::add(&mut registry, offline, EvidenceReview::new());
        Self::add(&mut registry, offline, LearningReview::new());
        Self::add(&mut registry, offline, LinguisticAnalysis::new());
        Self::add(&mut registry, offline, RepositoryDiscovery::new());
        Self::add(&mut registry, offline, CargoObservation::new());
        Self::add(&mut registry, offline, ReleaseObservation::new());
        Self::add(&mut registry, offline, SourceReferenceObservation::new());
        Self::add(&mut registry, offline, DocumentationObservation::new());

        registry
    }

    fn add(registry: &mut ModuleRegistry, offline: bool, module: impl CognitiveModule + 'static) {
        if !(offline && module.descriptor().is_external()) {
            registry.register(module);
        }
    }
}
