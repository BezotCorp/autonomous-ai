use std::time::Instant;

use crate::{
    engine::{disposition::Disposition, modulation::Modulation},
    individuality::{
        channel::Channel, channel_law::ChannelLaw, constitution::Constitution,
        dispositions::Dispositions,
    },
    latent::{
        latent_dynamics::LatentDynamics, latent_state::LatentState, occurrence::Occurrence,
        reaction_catalog::ReactionCatalog,
    },
    living::{
        clock_mode::ClockMode, decision::Decision, life_clock::LifeClock, life_event::LifeEvent,
        timeline::Timeline,
    },
    mathematics::random_source::RandomSource,
};

/// The living individual as the mechanisms meet it: its latent state
/// under its laws. It is the coupling between the latent space and the
/// cognition, in both directions:
///
/// - channels read the state (`channel`, `decision`, `modulation`), so that
///   every mechanism using them depends on it;
/// - occurrences move the state through the individual reaction laws
///   (`feel`), and time moves it through the latent dynamics (`advance`);
/// - both leave slow traces in the dispositions (plasticity).
pub(crate) struct Organism<'a> {
    constitution: &'a Constitution,
    dispositions: &'a mut Dispositions,
    latent: &'a mut LatentState,
    catalog: &'a mut ReactionCatalog,
    timeline: &'a mut Timeline,
    clock: &'a mut LifeClock,
    random: &'a mut RandomSource,
    mode: ClockMode,
    last: Instant,
}

impl<'a> Organism<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        constitution: &'a Constitution,
        dispositions: &'a mut Dispositions,
        latent: &'a mut LatentState,
        catalog: &'a mut ReactionCatalog,
        timeline: &'a mut Timeline,
        clock: &'a mut LifeClock,
        random: &'a mut RandomSource,
        mode: ClockMode,
    ) -> Self {
        Self {
            constitution,
            dispositions,
            latent,
            catalog,
            timeline,
            clock,
            random,
            mode,
            last: Instant::now(),
        }
    }

    pub(crate) fn random(&mut self) -> &mut RandomSource {
        self.random
    }

    pub(crate) fn record(&mut self, event: LifeEvent) {
        let lived = self.clock.lived();

        self.timeline.record(lived, event);
    }

    /// Lets time pass: the latent dynamics with its fluctuations, and the
    /// slow plastic changes that depend on the state lived through.
    pub(crate) fn advance(&mut self, seconds: f64) {
        if !(seconds.is_finite() && seconds > 0.0) {
            return;
        }

        let repairs = self.latent.repairs();

        LatentDynamics::advance(
            self.latent,
            self.constitution,
            self.dispositions,
            seconds,
            Some(self.random),
        );
        self.dispositions
            .consolidate(self.constitution, self.latent.pre(), seconds / 3_600.0);
        self.clock.advance(seconds);
        self.last = Instant::now();

        if self.latent.repairs() > repairs {
            self.record(LifeEvent::TechnicalError {
                detail: format!(
                    "{} non-finite latent values repaired",
                    self.latent.repairs() - repairs
                ),
            });
        }
    }

    /// Lived time elapsed since the organism last took time into account,
    /// in real mode; nothing in virtual mode.
    pub(crate) fn catch_up(&mut self) {
        if self.mode == ClockMode::Real {
            let elapsed = self.last.elapsed().as_secs_f64();

            self.advance(elapsed);
        }
    }

    /// Value of a channel in the current state.
    pub(crate) fn channel(&self, channel: Channel) -> f64 {
        let (bias, latent) =
            self.dispositions
                .activation(self.constitution, channel, &self.latent.activation());

        ChannelLaw::shape(channel.form(), bias + latent)
    }

    /// Rate of a transition through a rate channel, with the logarithm of
    /// a contextual factor (a salience, for instance), decomposed.
    pub(crate) fn decision(&self, channel: Channel, context: f64) -> Decision {
        let (bias, latent) =
            self.dispositions
                .activation(self.constitution, channel, &self.latent.activation());

        Decision::new(channel, bias, latent, context)
    }

    /// Habit: the mechanism behind a channel was engaged, and the state
    /// went from `before` to the current one.
    pub(crate) fn engage(&mut self, channel: Channel, before: &[f64]) {
        let after = self.latent.activation();

        self.dispositions
            .engage(self.constitution, channel, before, &after);
    }

    /// Records the current state in the timeline.
    pub(crate) fn sample(&mut self) {
        let state = self.latent.activation();

        self.record(LifeEvent::Latent { state });
    }

    fn react(&mut self, occurrence: &Occurrence) {
        let law = self.constitution.reaction_law(occurrence.signature());
        let latent = self.latent.activation();
        let gain = self.dispositions.gain(occurrence.signature());
        let impulse = law.impulse(&latent, occurrence.magnitude(), gain);
        let fallback = self.dispositions.baseline(self.constitution);
        let norm = impulse
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt();

        self.latent.shift(&impulse, &fallback);
        self.dispositions
            .repeat(self.constitution, occurrence.signature(), &law, &latent);
        self.catalog
            .record(occurrence.signature(), &impulse, self.clock.lived());
        self.record(LifeEvent::Occurrence {
            signature: occurrence.signature().to_owned(),
            magnitude: occurrence.magnitude(),
            impulse: if norm.is_finite() { norm } else { 0.0 },
        });
    }
}

impl Disposition for Organism<'_> {
    fn modulation(&self) -> Modulation {
        let weights = Channel::IMAGINATION.map(|channel| self.channel(channel));

        Modulation::new(
            self.channel(Channel::SamplingTemperature),
            self.channel(Channel::CostSensitivity),
            self.channel(Channel::InfluenceAdmission),
            self.channel(Channel::Abandonment),
            self.channel(Channel::SliceLength),
            self.channel(Channel::SceneContinuation),
            weights,
        )
    }

    fn feel(&mut self, occurrence: Occurrence) {
        match self.mode {
            ClockMode::Real => self.catch_up(),
            ClockMode::Virtual if occurrence.family() == "invoke" => {
                self.advance(ClockMode::VIRTUAL_INVOCATION_SECONDS);
            }
            ClockMode::Virtual => {}
        }

        self.react(&occurrence);
    }
}
