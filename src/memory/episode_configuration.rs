use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    acquisition::access_mode::AccessMode,
    engine::{learning_parameters::LearningParameters, modulation::Modulation},
    memory::{
        activity::Activity, imagined_capability_spec::ImaginedCapabilitySpec,
        imagined_element::ImaginedElement,
    },
};

/// Conditions under which an episode ran, needed to compare experiments.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct EpisodeConfiguration {
    seed: u64,
    acquired_knowledge: bool,
    learned_routing: bool,
    cost_weight_per_second: f64,
    scarcity_weight: f64,
    minimum_utility: f64,
    resource_budgets: BTreeMap<String, f64>,
    activity: Activity,
    access: AccessMode,
    excluded_modules: BTreeSet<String>,
    acquisition_cache: bool,
    influences: bool,
    excluded_influences: BTreeSet<uuid::Uuid>,
    maximum_invocations: Option<usize>,
    imagined_elements: Vec<ImaginedElement>,
    imagined_capabilities: Vec<ImaginedCapabilitySpec>,
    self_observations: Vec<(crate::representation::statement::Statement, String)>,
    revision: Option<String>,
    modulation: Modulation,
}

impl EpisodeConfiguration {
    pub(crate) fn new(seed: u64, acquired_knowledge: bool, learned_routing: bool) -> Self {
        Self {
            seed,
            acquired_knowledge,
            learned_routing,
            cost_weight_per_second: 0.0,
            scarcity_weight: 0.0,
            minimum_utility: 0.0,
            resource_budgets: BTreeMap::new(),
            activity: Activity::Action,
            access: AccessMode::Live,
            excluded_modules: BTreeSet::new(),
            acquisition_cache: true,
            influences: true,
            excluded_influences: BTreeSet::new(),
            maximum_invocations: None,
            imagined_elements: Vec::new(),
            imagined_capabilities: Vec::new(),
            self_observations: Vec::new(),
            revision: None,
            modulation: Modulation::neutral(),
        }
    }

    /// What introspection perceives of the individual itself, each
    /// statement with the aspect of the self it describes (its scope), at
    /// a revision of the self (so that a newer perception supersedes an
    /// older one in memory).
    pub(crate) fn perceiving_self(
        mut self,
        observations: Vec<(crate::representation::statement::Statement, String)>,
        revision: String,
    ) -> Self {
        self.self_observations = observations;
        self.revision = Some(revision);
        self
    }

    pub(crate) fn self_observations(
        &self,
    ) -> (
        &[(crate::representation::statement::Statement, String)],
        Option<&str>,
    ) {
        (&self.self_observations, self.revision.as_deref())
    }

    /// Records the modulation the episode started with.
    pub(crate) fn modulated(mut self, modulation: Modulation) -> Self {
        self.modulation = modulation;
        self
    }

    /// Records the decision parameters the episode ran with.
    pub(crate) fn with_parameters(mut self, parameters: &LearningParameters) -> Self {
        self.cost_weight_per_second = parameters.cost_weight_per_second();
        self.scarcity_weight = parameters.scarcity_weight();
        self.minimum_utility = parameters.minimum_utility();
        self
    }

    /// Limits imposed on resources for the episode, on top of their own
    /// constraints (to study scarcity without exhausting real quotas).
    pub(crate) fn with_resource_budgets(mut self, budgets: BTreeMap<String, f64>) -> Self {
        self.resource_budgets = budgets;
        self
    }

    pub(crate) fn with_activity(mut self, activity: Activity) -> Self {
        self.activity = activity;
        self
    }

    /// Replays recorded data instead of reaching the environment.
    pub(crate) fn with_access(mut self, access: AccessMode) -> Self {
        self.access = access;
        self
    }

    /// Capabilities the episode must do without.
    pub(crate) fn without_modules(mut self, modules: BTreeSet<String>) -> Self {
        self.excluded_modules = modules;
        self
    }

    /// Disables the reuse of acquired data, to tell technical gains apart
    /// from cognitive ones.
    pub(crate) fn without_acquisition_cache(mut self) -> Self {
        self.acquisition_cache = false;
        self
    }

    /// Leaves the context of the episode free of any influence of the
    /// unconscious (dreams, reflection, or the conscious alone).
    pub(crate) fn without_influences(mut self) -> Self {
        self.influences = false;
        self
    }

    /// Leaves out particular influences (to test what they change).
    pub(crate) fn excluding_influences(mut self, excluded: BTreeSet<uuid::Uuid>) -> Self {
        self.excluded_influences = excluded;
        self
    }

    pub(crate) fn with_maximum_invocations(mut self, maximum: usize) -> Self {
        self.maximum_invocations = Some(maximum);
        self
    }

    /// Setting of a dream scene: imagined elements and capabilities.
    pub(crate) fn imagining(
        mut self,
        elements: Vec<ImaginedElement>,
        capabilities: Vec<ImaginedCapabilitySpec>,
    ) -> Self {
        self.imagined_elements = elements;
        self.imagined_capabilities = capabilities;
        self
    }

    pub(crate) fn imagined_elements(&self) -> &[ImaginedElement] {
        &self.imagined_elements
    }

    pub(crate) fn imagined_capabilities(&self) -> &[ImaginedCapabilitySpec] {
        &self.imagined_capabilities
    }

    pub(crate) fn influenced(&self) -> bool {
        self.influences
    }

    pub(crate) fn excludes_influence(&self, influence: uuid::Uuid) -> bool {
        self.excluded_influences.contains(&influence)
    }

    pub(crate) fn maximum_invocations(&self) -> Option<usize> {
        self.maximum_invocations
    }

    pub(crate) fn activity(&self) -> Activity {
        self.activity
    }

    pub(crate) fn access(&self) -> AccessMode {
        self.access
    }

    pub(crate) fn excludes(&self, module: &str) -> bool {
        self.excluded_modules.contains(module)
    }

    pub(crate) fn acquisition_cache(&self) -> bool {
        self.acquisition_cache
    }

    pub(crate) fn resource_budgets(&self) -> &BTreeMap<String, f64> {
        &self.resource_budgets
    }

    /// Label grouping comparable episodes.
    pub(crate) fn label(&self) -> String {
        let budgets: Vec<String> = self
            .resource_budgets
            .iter()
            .map(|(resource, units)| format!("{resource}={units}"))
            .collect();

        format!(
            "activity={} knowledge={} routing={} cache={} influences={} cost_weight={} scarcity_weight={} minimum_utility={}{}",
            self.activity.label(),
            self.acquired_knowledge,
            self.learned_routing,
            self.acquisition_cache,
            self.influences,
            self.cost_weight_per_second,
            self.scarcity_weight,
            self.minimum_utility,
            if budgets.is_empty() {
                String::new()
            } else {
                format!(" budgets={}", budgets.join(","))
            }
        )
    }

    pub(crate) fn seed(&self) -> u64 {
        self.seed
    }

    pub(crate) fn acquired_knowledge(&self) -> bool {
        self.acquired_knowledge
    }

    pub(crate) fn learned_routing(&self) -> bool {
        self.learned_routing
    }
}
