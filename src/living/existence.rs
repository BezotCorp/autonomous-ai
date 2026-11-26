use serde::{Deserialize, Serialize};

use crate::{
    individuality::{
        birth_parameters::BirthParameters, constitution::Constitution, dispositions::Dispositions,
        identity::Identity,
    },
    latent::{latent_state::LatentState, reaction_catalog::ReactionCatalog},
    living::{
        activity_register::ActivityRegister, clock_mode::ClockMode,
        concern_register::ConcernRegister, life_clock::LifeClock, life_event::LifeEvent,
        organism::Organism, stimulus::Stimulus, timeline::Timeline, vigilance::Vigilance,
    },
    mathematics::{random_source::RandomSource, stable_hash::StableHash},
    memory::persistent_memory::PersistentMemory,
};

/// Everything an individual is and has lived, kept between executions:
/// its identity and birth, its born laws and what plasticity made of them,
/// its current latent state, its cognitive memory, its concerns, its
/// activities under way or suspended, the stimuli it has not dealt with,
/// the register of what happened to it, and its timeline. It is saved as
/// a whole, atomically, so that its parts never disagree.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Existence {
    identity: Identity,
    constitution: Constitution,
    dispositions: Dispositions,
    latent: LatentState,
    clock: LifeClock,
    vigilance: Vigilance,
    random: RandomSource,
    memory: PersistentMemory,
    concerns: ConcernRegister,
    activities: ActivityRegister,
    stimuli: Vec<Stimulus>,
    catalog: ReactionCatalog,
    timeline: Timeline,
}

impl Existence {
    /// Birth: the constitution is drawn from the birth parameters, the
    /// latent state starts where the constitution puts it, and the random
    /// stream of the existence (distinct from the one of the birth) starts
    /// from the same seed.
    pub(crate) fn born(name: String, birth: BirthParameters) -> Self {
        let constitution = Constitution::generate(birth.clone());
        let dispositions = Dispositions::new(&constitution);
        let latent = LatentState::new(constitution.initial());
        let mut timeline = Timeline::default();

        timeline.record(0.0, LifeEvent::Born);

        Self {
            random: RandomSource::new(StableHash::of(birth.seed(), "existence")),
            identity: Identity::new(name, birth),
            constitution,
            dispositions,
            latent,
            clock: LifeClock::new(),
            vigilance: Vigilance::Awake,
            memory: PersistentMemory::default(),
            concerns: ConcernRegister::default(),
            activities: ActivityRegister::default(),
            stimuli: Vec::new(),
            catalog: ReactionCatalog::default(),
            timeline,
        }
    }

    /// An exact copy under a new identity; with a new seed, its future
    /// random stream differs from the original's.
    pub(crate) fn duplicate(&self, name: String, reseed: Option<u64>) -> Self {
        let mut copy = self.clone();

        copy.identity = self.identity.duplicate(name);

        if let Some(seed) = reseed {
            copy.random = RandomSource::new(StableHash::of(seed, "existence"));
        }

        copy.timeline.record(
            copy.clock.lived(),
            LifeEvent::Intervention {
                description: format!(
                    "duplicated from {} ({}){}",
                    self.identity.name(),
                    self.identity.id(),
                    reseed
                        .map(|seed| format!(", reseeded {seed}"))
                        .unwrap_or_default()
                ),
            },
        );
        copy
    }

    pub(crate) fn identity(&self) -> &Identity {
        &self.identity
    }

    pub(crate) fn constitution(&self) -> &Constitution {
        &self.constitution
    }

    pub(crate) fn constitution_mut(&mut self) -> &mut Constitution {
        &mut self.constitution
    }

    pub(crate) fn dispositions(&self) -> &Dispositions {
        &self.dispositions
    }

    pub(crate) fn latent(&self) -> &LatentState {
        &self.latent
    }

    pub(crate) fn latent_mut(&mut self) -> &mut LatentState {
        &mut self.latent
    }

    pub(crate) fn clock(&self) -> &LifeClock {
        &self.clock
    }

    pub(crate) fn clock_mut(&mut self) -> &mut LifeClock {
        &mut self.clock
    }

    pub(crate) fn vigilance(&self) -> Vigilance {
        self.vigilance
    }

    pub(crate) fn set_vigilance(&mut self, vigilance: Vigilance) {
        self.vigilance = vigilance;
    }

    pub(crate) fn memory(&self) -> &PersistentMemory {
        &self.memory
    }

    pub(crate) fn memory_mut(&mut self) -> &mut PersistentMemory {
        &mut self.memory
    }

    pub(crate) fn concerns(&self) -> &ConcernRegister {
        &self.concerns
    }

    pub(crate) fn concerns_mut(&mut self) -> &mut ConcernRegister {
        &mut self.concerns
    }

    pub(crate) fn activities(&self) -> &ActivityRegister {
        &self.activities
    }

    pub(crate) fn activities_mut(&mut self) -> &mut ActivityRegister {
        &mut self.activities
    }

    pub(crate) fn stimuli(&self) -> &[Stimulus] {
        &self.stimuli
    }

    pub(crate) fn stimuli_mut(&mut self) -> &mut Vec<Stimulus> {
        &mut self.stimuli
    }

    pub(crate) fn catalog(&self) -> &ReactionCatalog {
        &self.catalog
    }

    pub(crate) fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    pub(crate) fn record(&mut self, event: LifeEvent) {
        self.timeline.record(self.clock.lived(), event);
    }

    /// The organism and the cognitive memory, borrowed together so that
    /// the engine can reason on the memory under the organism's state.
    pub(crate) fn organism(&mut self, mode: ClockMode) -> (Organism<'_>, &mut PersistentMemory) {
        (
            Organism::new(
                &self.constitution,
                &mut self.dispositions,
                &mut self.latent,
                &mut self.catalog,
                &mut self.timeline,
                &mut self.clock,
                &mut self.random,
                mode,
            ),
            &mut self.memory,
        )
    }
}
