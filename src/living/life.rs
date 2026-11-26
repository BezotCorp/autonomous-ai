use std::{collections::BTreeSet, error::Error, time::Instant};

use crate::{
    individuality::channel_law::ChannelLaw, living::session::Session, world::portal::Portal,
};

use uuid::Uuid;

use crate::{
    engine::{dreaming::Dreaming, imagination::Imagination, intelligence::Intelligence},
    events::event::Event,
    individuality::channel::Channel,
    latent::{latent_dynamics::LatentDynamics, occurrence::Occurrence},
    living::{
        activity_ending::ActivityEnding, activity_focus::ActivityFocus,
        activity_origin::ActivityOrigin, clock_mode::ClockMode, concern_register::ConcernRegister,
        decision::Decision, ended_activity::EndedActivity, existence::Existence,
        existence_store::ExistenceStore, life_event::LifeEvent, life_settings::LifeSettings,
        mailbox::Mailbox, ongoing_activity::OngoingActivity, response::Response,
        stimulus::Stimulus, transition::Transition, vigilance::Vigilance,
    },
    memory::{
        activity::Activity, answer_status::AnswerStatus,
        episode_configuration::EpisodeConfiguration, episode_record::EpisodeRecord,
    },
    representation::{statement::Statement, term::Term},
};

/// The continuous existence of an individual. There is no workflow: at
/// every moment, the activities under way advance by slices, and every
/// possible change (acting on a stimulus, beginning an activity, resuming
/// one, falling asleep, waking, a concern surfacing) happens at a rate
/// read from the latent state through the individual's channels, in
/// competition with the others (competing exponential waiting times).
/// Activities in progress may be interrupted, may drift into another kind,
/// or may be abandoned by the reasoner, at rates and probabilities read
/// the same way. When nothing is under way, time passes (really, or
/// virtually) until the next change; the latent state keeps evolving.
///
/// Structural constraints only: one conscious activity runs at a time
/// (starting another one suspends it), the unconscious runs at most
/// `MAXIMUM_UNCONSCIOUS` activities besides, sleep closes the conscious,
/// dreams need sleep. Activities interleave slice by slice on one thread,
/// so that the memory stays coherent and an existence is reproducible.
pub(crate) struct Life<'a> {
    intelligence: &'a Intelligence,
    settings: LifeSettings,
    mailbox: Mailbox,
    portal: Option<Portal>,
}

/// What a slice of activity came to.
#[derive(Default)]
struct SliceOutcome {
    summary: String,
    ending: Option<ActivityEnding>,
    episode: Option<Uuid>,
    /// An activity the slice deliberately takes up next.
    follow: Option<OngoingActivity>,
}

impl<'a> Life<'a> {
    const MAXIMUM_UNCONSCIOUS: usize = 2;
    /// Invocations of one slice of reasoning (before modulation).
    const SLICE_INVOCATIONS: usize = 12;
    /// Technical bound on the slices of one activity.
    const MAXIMUM_SLICES: u32 = 64;
    /// Longest lapse over which rates are considered constant.
    const RATE_WINDOW: f64 = 60.0;
    /// Shortest lived duration of a slice, so that time always advances.
    const MINIMUM_SLICE: f64 = 1.0;
    /// Lived seconds between two samples of the latent state.
    const SAMPLING: f64 = 60.0;
    /// Bound on the concerns a single imaginative unfolding may raise.
    const IMAGINED_CONCERNS: usize = 3;
    /// Salience a stimulus has per question it asks (plus one for itself),
    /// before the individual's stimulus gain.
    const STIMULUS_SALIENCE: f64 = 1.0;
    /// Saliences given to concerns by origin (before the concern gain):
    /// orders of magnitude, not measured values.
    const IMPASSE_SALIENCE: f64 = 1.0;
    const UNFINISHED_SALIENCE: f64 = 1.5;
    const EXAMINATION_SALIENCE: f64 = 0.5;
    const IMAGINATION_SALIENCE: f64 = 0.3;
    /// Exponent of the concern gain applied to the salience of a concern
    /// the individual attends to: `s ← s · gain^ATTENTION`. With a gain
    /// above 1 (in the current state) attention intensifies the concern,
    /// below 1 it appeases it; neither is the rule.
    const ATTENTION: f64 = 0.5;
    const ECHO_SALIENCE: f64 = 0.2;

    pub(crate) fn new(intelligence: &'a Intelligence, settings: LifeSettings) -> Self {
        let mailbox = Mailbox::new(settings.directory());

        Self {
            intelligence,
            settings,
            mailbox,
            portal: None,
        }
    }

    /// Present in an environment, through this portal: messages reach the
    /// existence as stimuli, and its capabilities may perceive and emit.
    pub(crate) fn with_portal(mut self, portal: Portal) -> Self {
        self.portal = Some(portal);
        self
    }

    /// Lives for the session's duration, or until a technical stop is
    /// requested, saving at checkpoints and at the end.
    pub(crate) fn live(
        &self,
        existence: &mut Existence,
        store: &ExistenceStore,
    ) -> Result<(), Box<dyn Error>> {
        let mut session = self.begin_session(existence);
        let end = session.end();

        self.run(existence, store, &mut session, end)?;
        self.end_session(existence, store)
    }

    /// Opens a session (see `open`).
    pub(crate) fn begin_session(&self, existence: &mut Existence) -> Session {
        self.open(existence);

        let start = existence.clock().lived();

        Session::new(start, self.settings.duration())
    }

    /// Closes a session and saves the existence.
    pub(crate) fn end_session(
        &self,
        existence: &mut Existence,
        store: &ExistenceStore,
    ) -> Result<(), Box<dyn Error>> {
        self.sample(existence);
        existence.record(LifeEvent::SessionClosed);
        store.save(existence)
    }

    /// Lives until the lived time `until` (at most the end of the session),
    /// or a technical stop. Returns whether the session may go on.
    pub(crate) fn run(
        &self,
        existence: &mut Existence,
        store: &ExistenceStore,
        session: &mut Session,
        until: f64,
    ) -> Result<bool, Box<dyn Error>> {
        let until = until.min(session.end());

        loop {
            if session.is_stopped() {
                return Ok(false);
            }

            if self.mailbox.stop_requested() {
                self.mailbox.clear_stop()?;
                existence.record(LifeEvent::Intervention {
                    description: "technical stop requested".to_owned(),
                });
                session.stop();
                return Ok(false);
            }

            let now = existence.clock().lived();

            if now >= until {
                return Ok(now < session.end());
            }

            while let Some(delivery) = self.settings.schedule().get(session.scheduled())
                && session.start() + delivery.at() <= now
            {
                self.receive(existence, delivery.event().clone());
                session.delivered_scheduled();
            }

            let (events, errors) = self.mailbox.collect();

            for error in errors {
                existence.record(LifeEvent::TechnicalError { detail: error });
            }

            for event in events {
                self.receive(existence, event);
            }

            self.exchange(existence);

            let running: Vec<Uuid> = existence
                .activities()
                .running()
                .map(OngoingActivity::id)
                .collect();

            if running.is_empty() {
                let next_delivery = self
                    .settings
                    .schedule()
                    .get(session.scheduled())
                    .map_or(f64::INFINITY, |delivery| {
                        session.start() + delivery.at() - now
                    });
                let horizon = [
                    Self::RATE_WINDOW,
                    until - now,
                    next_delivery,
                    if self.settings.clock() == ClockMode::Real {
                        1.0
                    } else {
                        f64::INFINITY
                    },
                    // The next message known to be on its way.
                    self.portal.as_ref().and_then(Portal::next_arrival).map_or(
                        f64::INFINITY,
                        |arrival| {
                            self.portal
                                .as_ref()
                                .map_or(f64::INFINITY, |portal| portal.lived_at(arrival) - now)
                        },
                    ),
                ]
                .into_iter()
                .fold(f64::INFINITY, f64::min)
                .max(0.0);

                self.wait(existence, horizon)?;
            } else {
                let id = running[session.next_turn() % running.len()];
                let lived = self.slice(existence, id)?;

                self.fade(existence, lived);
                self.compete(existence, lived);
            }

            self.exchange(existence);

            let now = existence.clock().lived();

            if session.sample_due(now, Self::SAMPLING) {
                self.sample(existence);
            }

            if session.checkpoint_due(self.settings.checkpoint()) {
                store.save(existence)?;
            }
        }
    }

    /// Records the latent state and the plastic changes so far.
    fn sample(&self, existence: &mut Existence) {
        existence.organism(self.settings.clock()).0.sample();

        let latent = existence.latent().activation();
        let channels = Channel::ALL
            .iter()
            .map(|channel| {
                let (bias, term) = existence.dispositions().activation(
                    existence.constitution(),
                    *channel,
                    &latent,
                );

                ChannelLaw::shape(channel.form(), bias + term)
            })
            .collect();
        let dispositions = existence.dispositions();
        let coupling_change = dispositions.coupling_change().frobenius();
        let baseline_change = dispositions
            .baseline_change()
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt();
        let engagements = dispositions.engagements();

        existence.record(LifeEvent::Plasticity {
            channels,
            coupling_change,
            baseline_change,
            engagements,
        });
    }

    /// With the environment: follows its time, records what the existence
    /// emitted, and receives what reached it (once only, whatever the
    /// interruptions of the processes).
    fn exchange(&self, existence: &mut Existence) {
        let Some(portal) = &self.portal else {
            return;
        };

        portal.follow(existence.clock().lived());

        for (transmission, to, status) in portal.take_emitted() {
            existence.record(LifeEvent::Transmitted {
                transmission,
                to,
                status: format!("{status:?}"),
            });
        }

        for transmission in portal.receive() {
            let id = transmission.id().to_string();
            let known = existence.timeline().entries().iter().any(|entry| {
                matches!(entry.event(), LifeEvent::StimulusReceived { event, .. } if event == &id)
            });

            if known {
                continue;
            }

            let occurred_at = chrono::Utc::now();
            let source = format!("{}{}", Event::INDIVIDUAL_SOURCE, transmission.from());
            let message = transmission.message();

            self.receive(
                existence,
                Event::transmitted(
                    id.clone(),
                    source.clone(),
                    occurred_at,
                    message.statements().to_vec(),
                    message.documents().to_vec(),
                    message.questions().to_vec(),
                ),
            );

            // What the sender had itself been told keeps its chain of
            // sources: one perception per chain.
            let chains: BTreeSet<&String> =
                message.relayed().iter().map(|(_, chain)| chain).collect();

            for (index, chain) in chains.into_iter().enumerate() {
                self.receive(
                    existence,
                    Event::transmitted(
                        format!("{id}/relayed/{index}"),
                        format!("{source} relaying {chain}"),
                        occurred_at,
                        message
                            .relayed()
                            .iter()
                            .filter(|(_, from)| from == chain)
                            .map(|(statement, _)| statement.clone())
                            .collect(),
                        Vec::new(),
                        Vec::new(),
                    ),
                );
            }
        }
    }

    /// Opens a session: the time elapsed while the process was stopped is
    /// applied to the latent state, deterministically (relaxation and
    /// internal coupling, without fluctuations, occurrences or plasticity:
    /// nothing was lived, but time passed).
    fn open(&self, existence: &mut Existence) {
        let wall_gap = existence.clock_mut().open_session();
        let dormancy = self.settings.dormancy(wall_gap).max(0.0);

        if dormancy > 0.0 {
            let constitution = existence.constitution().clone();
            let dispositions = existence.dispositions().clone();

            LatentDynamics::advance(
                existence.latent_mut(),
                &constitution,
                &dispositions,
                dormancy,
                None,
            );
            existence.clock_mut().sleep_through(dormancy);
        }

        existence.record(LifeEvent::SessionOpened {
            wall_gap,
            dormancy_applied: dormancy,
        });

        if !self.settings.excluded().is_empty() || !self.settings.influences() {
            existence.record(LifeEvent::Intervention {
                description: format!(
                    "session without {}{}",
                    self.settings
                        .excluded()
                        .iter()
                        .map(|activity| activity.label())
                        .collect::<Vec<_>>()
                        .join(", "),
                    if self.settings.influences() {
                        ""
                    } else {
                        " (the conscious ignoring influences)"
                    }
                ),
            });
        }
    }

    fn feel(&self, existence: &mut Existence, signature: impl Into<String>, magnitude: f64) {
        use crate::engine::disposition::Disposition;

        existence
            .organism(self.settings.clock())
            .0
            .feel(Occurrence::new(signature, magnitude));
    }

    fn channel(&self, existence: &mut Existence, channel: Channel) -> f64 {
        existence.organism(self.settings.clock()).0.channel(channel)
    }

    /// A stimulus arrives: it is perceived with a salience the individual's
    /// state weighs, and it moves that state; what follows is not decided
    /// here.
    fn receive(&self, existence: &mut Existence, event: Event) {
        let gain = self.channel(existence, Channel::StimulusGain);
        let salience = Self::STIMULUS_SALIENCE * (1.0 + event.questions().len() as f64) * gain;
        let lived = existence.clock().lived();
        let stimulus = Stimulus::new(event, lived, salience);
        let predicates: Vec<String> = stimulus
            .event()
            .questions()
            .iter()
            .map(|question| question.predicate().to_owned())
            .collect();

        existence.record(LifeEvent::StimulusReceived {
            stimulus: stimulus.id(),
            event: stimulus.event().id().to_owned(),
            salience: stimulus.salience(),
        });

        let signature = format!("stimulus:{}", stimulus.event().source());

        existence.stimuli_mut().push(stimulus);
        self.feel(existence, signature, 1.0);

        let concern_gain = self.channel(existence, Channel::ConcernGain);

        existence
            .concerns_mut()
            .reactivate(&predicates, Self::ECHO_SALIENCE * concern_gain, lived);
    }

    /// Lets time pass while nothing is under way, until the next change or
    /// the horizon.
    fn wait(&self, existence: &mut Existence, horizon: f64) -> Result<(), Box<dyn Error>> {
        let candidates = self.candidates(existence);
        let total: f64 = candidates
            .iter()
            .map(|(_, decision)| decision.rate() / 3_600.0)
            .sum();
        let delay = existence
            .organism(self.settings.clock())
            .0
            .random()
            .exponential(total);
        let lapse = delay.min(horizon).max(0.0);

        match self.settings.clock() {
            ClockMode::Real => {
                let started = Instant::now();

                std::thread::sleep(std::time::Duration::from_secs_f64(lapse.min(1.0)));
                existence
                    .organism(ClockMode::Real)
                    .0
                    .advance(started.elapsed().as_secs_f64());
            }
            ClockMode::Virtual => {
                // At least a second, so that lived time always advances.
                existence
                    .organism(ClockMode::Virtual)
                    .0
                    .advance(lapse.max(Self::MINIMUM_SLICE.min(horizon.max(0.0))));
            }
        }

        self.fade(existence, lapse);

        if delay <= horizon {
            self.fire(existence, candidates);
        }

        Ok(())
    }

    /// After a slice that lasted `lived` seconds, whether another change
    /// happened meanwhile (one at most).
    fn compete(&self, existence: &mut Existence, lived: f64) {
        let candidates = self.candidates(existence);
        let total: f64 = candidates
            .iter()
            .map(|(_, decision)| decision.rate() / 3_600.0)
            .sum();
        let delay = existence
            .organism(self.settings.clock())
            .0
            .random()
            .exponential(total);

        if delay <= lived {
            self.fire(existence, candidates);
        }
    }

    fn fire(&self, existence: &mut Existence, candidates: Vec<(Transition, Decision)>) {
        let weights: Vec<f64> = candidates
            .iter()
            .map(|(_, decision)| decision.rate())
            .collect();
        let Some(index) = existence
            .organism(self.settings.clock())
            .0
            .random()
            .weighted(&weights)
        else {
            return;
        };
        let Some((transition, decision)) = candidates.into_iter().nth(index) else {
            return;
        };

        self.apply(existence, transition, decision);
    }

    /// Every change that could happen now, with its rate.
    fn candidates(&self, existence: &mut Existence) -> Vec<(Transition, Decision)> {
        let mut candidates = Vec::new();
        let vigilance = existence.vigilance();
        let running: Vec<Activity> = existence
            .activities()
            .running()
            .map(OngoingActivity::activity)
            .collect();
        let unconscious_running = running
            .iter()
            .filter(|activity| !activity.is_conscious())
            .count();
        let stimuli: Vec<(Uuid, f64)> = existence
            .stimuli()
            .iter()
            .map(|stimulus| (stimulus.id(), stimulus.salience()))
            .collect();
        let concerns: Vec<(Uuid, Statement, f64, bool)> = existence
            .concerns()
            .active()
            .map(|concern| {
                (
                    concern.id(),
                    concern.subject().clone(),
                    concern.salience(),
                    concern.is_conscious(),
                )
            })
            .collect();
        let suspended: Vec<(Uuid, Activity, f64)> = existence
            .activities()
            .suspended()
            .map(|activity| {
                let concern = activity
                    .focus()
                    .concern()
                    .and_then(|id| existence.concerns().get(id))
                    .filter(|concern| concern.is_active())
                    .map_or(0.0, |concern| concern.salience());

                (
                    activity.id(),
                    activity.activity(),
                    activity.hold() + concern,
                )
            })
            .collect();
        let organism = existence.organism(self.settings.clock()).0;
        let allows =
            |activity: Activity| self.settings.allows(activity) && !running.contains(&activity);

        match vigilance {
            Vigilance::Awake => {
                if self.settings.allows(Activity::Action) {
                    for (id, salience) in &stimuli {
                        candidates.push((
                            Transition::Attend(*id),
                            organism.decision(Channel::Attend, salience.max(1e-9).ln()),
                        ));
                    }
                }

                for (activity, channel) in [
                    (Activity::Introspection, Channel::Introspect),
                    (Activity::Reverie, Channel::Reverie),
                ] {
                    if allows(activity) {
                        candidates.push((
                            Transition::Start(
                                activity,
                                Box::new(ActivityFocus::Free),
                                ActivityOrigin::Spontaneous,
                            ),
                            organism.decision(channel, 0.0),
                        ));
                    }
                }

                // The propensity to simulate grows with the total salience of
                // the conscious concerns, with diminishing returns
                // (ln(1 + total)); each concern is taken up in proportion to
                // its share. The same holds for concerns surfacing.
                let share = |conscious: bool, salience: f64| {
                    let total: f64 = concerns
                        .iter()
                        .filter(|concern| concern.3 == conscious)
                        .map(|concern| concern.2)
                        .sum();

                    (salience / total.max(f64::MIN_POSITIVE) * (1.0 + total).ln())
                        .max(1e-12)
                        .ln()
                };

                // A conscious concern may also be acted upon in reality (the
                // rate of acting, weighted by its share of salience).
                if allows(Activity::Action) {
                    for (id, subject, salience, conscious) in &concerns {
                        if *conscious {
                            candidates.push((
                                Transition::Start(
                                    Activity::Action,
                                    Box::new(ActivityFocus::Concern {
                                        id: *id,
                                        subject: subject.clone(),
                                    }),
                                    ActivityOrigin::Concern(*id),
                                ),
                                organism.decision(Channel::Attend, share(true, *salience)),
                            ));
                        }
                    }
                }

                if allows(Activity::Simulation) {
                    for (id, subject, salience, conscious) in &concerns {
                        if *conscious {
                            candidates.push((
                                Transition::Start(
                                    Activity::Simulation,
                                    Box::new(ActivityFocus::Concern {
                                        id: *id,
                                        subject: subject.clone(),
                                    }),
                                    ActivityOrigin::Concern(*id),
                                ),
                                organism.decision(Channel::Simulate, share(true, *salience)),
                            ));
                        }
                    }
                }

                for (id, _, salience, conscious) in &concerns {
                    if !*conscious {
                        candidates.push((
                            Transition::Surface(*id),
                            organism.decision(Channel::Surfacing, share(false, *salience)),
                        ));
                    }
                }

                candidates.push((
                    Transition::FallAsleep,
                    organism.decision(Channel::FallAsleep, 0.0),
                ));
            }
            Vigilance::Asleep => {
                let pressure: f64 = stimuli.iter().map(|(_, salience)| salience).sum();

                candidates.push((
                    Transition::Wake,
                    organism.decision(Channel::Wake, (1.0 + pressure).ln()),
                ));

                if allows(Activity::Dream) && unconscious_running < Self::MAXIMUM_UNCONSCIOUS {
                    candidates.push((
                        Transition::Start(
                            Activity::Dream,
                            Box::new(ActivityFocus::Free),
                            ActivityOrigin::Spontaneous,
                        ),
                        organism.decision(Channel::Dream, 0.0),
                    ));
                }
            }
        }

        if allows(Activity::Reflection) && unconscious_running < Self::MAXIMUM_UNCONSCIOUS {
            candidates.push((
                Transition::Start(
                    Activity::Reflection,
                    Box::new(ActivityFocus::Free),
                    ActivityOrigin::Spontaneous,
                ),
                organism.decision(Channel::Reflect, 0.0),
            ));
        }

        for (id, activity, hold) in suspended {
            if self.settings.allows(activity)
                && (vigilance == Vigilance::Awake || !activity.is_conscious())
            {
                candidates.push((
                    Transition::Resume(id),
                    organism.decision(Channel::Resume, hold.max(1e-9).ln()),
                ));
            }
        }

        candidates
    }

    fn apply(&self, existence: &mut Existence, transition: Transition, decision: Decision) {
        match transition {
            Transition::Attend(stimulus) => {
                let position = existence
                    .stimuli()
                    .iter()
                    .position(|candidate| candidate.id() == stimulus);

                if let Some(position) = position {
                    let stimulus = existence.stimuli_mut().remove(position);
                    let lived = existence.clock().lived();

                    self.begin(
                        existence,
                        OngoingActivity::new(
                            Activity::Action,
                            ActivityOrigin::Stimulus(stimulus.id()),
                            ActivityFocus::Event(stimulus.event().clone()),
                            lived,
                        ),
                        Some(decision),
                    );
                }
            }
            Transition::Start(activity, focus, origin) => {
                let lived = existence.clock().lived();

                self.begin(
                    existence,
                    OngoingActivity::new(activity, origin, *focus, lived),
                    Some(decision),
                );
            }
            Transition::Resume(id) => {
                let Some(activity) = existence
                    .activities()
                    .get(id)
                    .map(OngoingActivity::activity)
                else {
                    return;
                };

                if activity.is_conscious() {
                    self.suspend_conscious(existence, "displaced by a resumption");
                }

                if let Some(ongoing) = existence.activities_mut().get_mut(id) {
                    ongoing.resume();
                }

                existence.record(LifeEvent::ActivityResumed {
                    activity: id,
                    decision,
                });
                self.feel(existence, format!("resume:{}", activity.label()), 1.0);
            }
            Transition::FallAsleep => {
                self.suspend_conscious(existence, "sleep");
                existence.set_vigilance(Vigilance::Asleep);
                existence.record(LifeEvent::Vigilance {
                    to: Vigilance::Asleep,
                    decision,
                });
                self.feel(existence, "vigilance:asleep", 1.0);
            }
            Transition::Wake => {
                existence.set_vigilance(Vigilance::Awake);
                existence.record(LifeEvent::Vigilance {
                    to: Vigilance::Awake,
                    decision,
                });
                self.feel(existence, "vigilance:awake", 1.0);
            }
            Transition::Surface(concern) => {
                if let Some(concern) = existence.concerns_mut().get_mut(concern) {
                    concern.surface();
                }

                existence.record(LifeEvent::ConcernSurfaced { concern, decision });
                self.feel(existence, "concern:surfaced", 1.0);
            }
        }
    }

    /// Begins an activity; a conscious one displaces the conscious activity
    /// under way, which is suspended.
    fn begin(
        &self,
        existence: &mut Existence,
        activity: OngoingActivity,
        decision: Option<Decision>,
    ) {
        if activity.activity().is_conscious() {
            self.suspend_conscious(existence, "displaced");
        }

        existence.record(LifeEvent::ActivityStarted {
            activity: activity.id(),
            kind: activity.activity(),
            origin: activity.origin().clone(),
            focus: activity.focus().describe(),
            decision,
        });

        let signature = format!("begin:{}", activity.activity().label());

        existence.activities_mut().start(activity);
        self.feel(existence, signature, 1.0);
    }

    fn suspend_conscious(&self, existence: &mut Existence, cause: &str) {
        let conscious: Vec<Uuid> = existence
            .activities()
            .running()
            .filter(|activity| activity.activity().is_conscious())
            .map(OngoingActivity::id)
            .collect();

        for id in conscious {
            if let Some(activity) = existence.activities_mut().get_mut(id) {
                activity.suspend();
            }

            existence.record(LifeEvent::ActivitySuspended {
                activity: id,
                cause: cause.to_owned(),
            });
            self.feel(existence, "suspend:activity", 1.0);
        }
    }

    /// Time passing fades concerns, waiting stimuli and suspended
    /// activities, at the rate the state gives.
    fn fade(&self, existence: &mut Existence, seconds: f64) {
        if !(seconds.is_finite() && seconds > 0.0) {
            return;
        }

        let rate = self.channel(existence, Channel::ConcernFading);
        let factor = (-rate * seconds / 3_600.0).exp();
        let faded = existence.concerns_mut().fade(factor);

        for concern in &faded {
            existence.record(LifeEvent::ConcernConcluded {
                concern: *concern,
                how: "faded".to_owned(),
            });
        }

        if !faded.is_empty() {
            self.feel(existence, "concern:faded", (1.0 + faded.len() as f64).ln());
        }

        for stimulus in existence.stimuli_mut().iter_mut() {
            stimulus.fade(factor);
        }

        let neglected: Vec<Stimulus> = {
            let stimuli = existence.stimuli_mut();
            let (neglected, kept) = std::mem::take(stimuli)
                .into_iter()
                .partition(|stimulus| stimulus.salience() < ConcernRegister::FADED);

            *stimuli = kept;
            neglected
        };

        for stimulus in neglected {
            existence.record(LifeEvent::StimulusNeglected {
                stimulus: stimulus.id(),
                event: stimulus.event().id().to_owned(),
            });
            self.feel(existence, "neglect:stimulus", 1.0);

            for question in stimulus.event().questions() {
                self.concern(
                    existence,
                    question.clone(),
                    format!("neglected:{}", stimulus.event().id()),
                    Self::UNFINISHED_SALIENCE,
                );
            }
        }

        let mut dissolved = Vec::new();

        for activity in existence.activities_mut().ongoing_mut() {
            if activity.is_suspended() {
                activity.fade_hold(factor);

                if activity.hold() < ConcernRegister::FADED {
                    dissolved.push(activity.id());
                }
            }
        }

        for id in dissolved {
            self.end(
                existence,
                id,
                ActivityEnding::Dissolved,
                "faded while suspended".to_owned(),
            );
        }
    }

    fn concern(
        &self,
        existence: &mut Existence,
        subject: Statement,
        origin: String,
        salience: f64,
    ) {
        let gain = self.channel(existence, Channel::ConcernGain);
        let lived = existence.clock().lived();
        let description = subject.to_string();
        let (id, new, displaced) =
            existence
                .concerns_mut()
                .form(subject, origin.clone(), salience * gain, lived);

        for concern in displaced {
            existence.record(LifeEvent::ConcernConcluded {
                concern,
                how: "displaced".to_owned(),
            });
        }

        if new {
            let salience = existence
                .concerns()
                .get(id)
                .map_or(0.0, |concern| concern.salience());

            existence.record(LifeEvent::ConcernFormed {
                concern: id,
                subject: description,
                origin: origin.clone(),
                salience,
            });
            self.feel(
                existence,
                format!("concern:{}", origin.split(':').next().unwrap_or("formed")),
                1.0,
            );
        } else {
            self.feel(existence, "concern:renewed", 1.0);
        }
    }

    /// One slice of an activity; returns the lived time it took.
    fn slice(&self, existence: &mut Existence, id: Uuid) -> Result<f64, Box<dyn Error>> {
        let Some(activity) = existence.activities().get(id).cloned() else {
            return Ok(0.0);
        };
        let before = existence.clock().lived();
        let latent_before = existence.latent().activation();
        let kind = activity.activity();
        let outcome = match kind {
            Activity::Action | Activity::Reflection | Activity::Introspection => {
                self.reason(existence, &activity)?
            }
            Activity::Simulation | Activity::Reverie | Activity::Dream => {
                self.imagine(existence, &activity)
            }
        };

        if self.settings.clock() == ClockMode::Virtual {
            let lived = existence.clock().lived() - before;

            if lived < Self::MINIMUM_SLICE {
                existence
                    .organism(ClockMode::Virtual)
                    .0
                    .advance(Self::MINIMUM_SLICE - lived);
            }
        } else {
            existence.organism(ClockMode::Real).0.catch_up();
        }

        let lived = existence.clock().lived() - before;

        if let Some(ongoing) = existence.activities_mut().get_mut(id) {
            ongoing.account(lived, outcome.episode);
        }

        existence
            .organism(self.settings.clock())
            .0
            .engage(Self::channel_of(kind), &latent_before);
        existence.record(LifeEvent::Slice {
            activity: id,
            kind,
            lived,
            summary: outcome.summary.clone(),
        });

        let ending = match outcome.ending {
            Some(ending) => Some(ending),
            None => self.hazards(existence, id, kind, lived),
        };

        match ending {
            Some(ActivityEnding::Transformed(into)) => {
                let focus = activity.focus().clone();

                self.end(
                    existence,
                    id,
                    ActivityEnding::Transformed(into),
                    outcome.summary,
                );

                let lived = existence.clock().lived();

                self.begin(
                    existence,
                    OngoingActivity::new(into, ActivityOrigin::Transformation(kind), focus, lived),
                    None,
                );
            }
            Some(ending) => self.end(existence, id, ending, outcome.summary),
            None => {}
        }

        if let Some(follow) = outcome.follow {
            let signature = format!("deliberate:{}", follow.activity().label());

            self.feel(existence, signature, 1.0);
            self.begin(existence, follow, None);
        }

        Ok(lived)
    }

    /// Whether the internal dynamics interrupts or transforms an activity
    /// that could go on, over the lived time of its last slice.
    fn hazards(
        &self,
        existence: &mut Existence,
        id: Uuid,
        kind: Activity,
        lived: f64,
    ) -> Option<ActivityEnding> {
        if existence
            .activities()
            .get(id)
            .is_some_and(|activity| activity.slices() >= Self::MAXIMUM_SLICES)
        {
            return Some(ActivityEnding::Bounded);
        }

        let hours = lived / 3_600.0;
        let vigilance = existence.vigilance();
        let targets: Vec<Activity> = match kind {
            Activity::Action => vec![Activity::Simulation, Activity::Reverie],
            Activity::Introspection => vec![Activity::Simulation, Activity::Reverie],
            Activity::Simulation => vec![Activity::Introspection, Activity::Reverie],
            Activity::Reverie => vec![Activity::Introspection, Activity::Simulation],
            Activity::Reflection if vigilance == Vigilance::Asleep => vec![Activity::Dream],
            Activity::Dream => vec![Activity::Reflection],
            Activity::Reflection => Vec::new(),
        }
        .into_iter()
        .filter(|target| {
            self.settings.allows(*target)
                && (vigilance == Vigilance::Awake || !target.is_conscious())
        })
        .collect();

        let mut organism = existence.organism(self.settings.clock()).0;
        let interrupt = organism.decision(Channel::Interrupt, 0.0).rate();
        let transform = if targets.is_empty() {
            0.0
        } else {
            organism.decision(Channel::Transform, 0.0).rate()
        };
        let weights: Vec<f64> = targets
            .iter()
            .map(|target| organism.decision(Self::channel_of(*target), 0.0).rate())
            .collect();
        let random = organism.random();

        if random.exponential(interrupt) <= hours {
            return Some(ActivityEnding::Interrupted);
        }

        if random.exponential(transform) <= hours {
            return random
                .weighted(&weights)
                .map(|index| ActivityEnding::Transformed(targets[index]));
        }

        None
    }

    fn end(&self, existence: &mut Existence, id: Uuid, ending: ActivityEnding, summary: String) {
        let Some(activity) = existence.activities_mut().remove(id) else {
            return;
        };
        let lived = existence.clock().lived();
        let kind = activity.activity();

        if kind == Activity::Action
            && ending != ActivityEnding::Completed
            && let ActivityFocus::Event(event) = activity.focus()
        {
            for question in event.questions() {
                self.concern(
                    existence,
                    question.clone(),
                    format!("unfinished:{}", event.id()),
                    Self::UNFINISHED_SALIENCE,
                );
            }
        }

        existence.record(LifeEvent::ActivityEnded {
            activity: id,
            kind,
            ending,
            summary: summary.clone(),
        });
        existence.activities_mut().finish(EndedActivity::new(
            id,
            kind,
            activity.origin().clone(),
            activity.focus().describe(),
            activity.started_at(),
            lived,
            activity.lived(),
            activity.slices(),
            ending,
            summary,
        ));
        self.feel(
            existence,
            format!("end:{}:{}", kind.label(), ending.label()),
            1.0,
        );
    }

    fn channel_of(activity: Activity) -> Channel {
        match activity {
            Activity::Action => Channel::Attend,
            Activity::Reflection => Channel::Reflect,
            Activity::Introspection => Channel::Introspect,
            Activity::Simulation => Channel::Simulate,
            Activity::Reverie => Channel::Reverie,
            Activity::Dream => Channel::Dream,
        }
    }

    /// Questions about itself the cognition can ask: what its reflective
    /// modules answer.
    fn reflective_questions(&self) -> Vec<Statement> {
        self.intelligence
            .descriptors()
            .filter(|descriptor| descriptor.reflects())
            .flat_map(|descriptor| descriptor.supplied().to_vec())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter_map(|signature| {
                let (predicate, arity) = signature.rsplit_once('/')?;
                let arity: usize = arity.parse().ok()?;

                Some(Statement::new(
                    predicate,
                    (0..arity)
                        .map(|index| Term::Variable(format!("x{index}")))
                        .collect(),
                ))
            })
            .collect()
    }

    /// What introspection perceives of the individual: anonymous sensations
    /// (projections of the latent state it cannot read directly), its
    /// conscious concerns, what it was doing lately, whether it is awake.
    fn self_observations(existence: &Existence) -> Vec<(Statement, String)> {
        let mut observations: Vec<(Statement, String)> = existence
            .latent()
            .sensations(existence.constitution())
            .into_iter()
            .map(|(index, level)| {
                (
                    Statement::new(
                        "sensation",
                        vec![Term::symbol(format!("s{index}")), Term::symbol(level)],
                    ),
                    format!("sensation:s{index}"),
                )
            })
            .collect();

        let mut conscious: Vec<_> = existence
            .concerns()
            .active()
            .filter(|concern| concern.is_conscious())
            .collect();

        conscious.sort_by(|left, right| right.salience().total_cmp(&left.salience()));

        for concern in conscious.into_iter().take(8) {
            let short = concern.id().simple().to_string()[..8].to_owned();

            observations.push((
                Statement::new(
                    "concern",
                    vec![
                        Term::symbol(format!("c{short}")),
                        Term::symbol(concern.subject().predicate()),
                    ],
                ),
                format!("concern:{}", concern.id()),
            ));
        }

        for (position, ended) in existence
            .activities()
            .ended()
            .iter()
            .rev()
            .take(5)
            .enumerate()
        {
            observations.push((
                Statement::new(
                    "recent_activity",
                    vec![
                        Term::symbol(format!("r{}", position + 1)),
                        Term::symbol(ended.activity().label()),
                        Term::symbol(ended.ending().label()),
                    ],
                ),
                format!("recent_activity:r{}", position + 1),
            ));
        }

        observations.push((
            Statement::new(
                "vigilance",
                vec![Term::symbol(existence.vigilance().label())],
            ),
            "vigilance".to_owned(),
        ));
        observations
    }

    /// A slice of reasoning: an episode of the engine under the organism.
    fn reason(
        &self,
        existence: &mut Existence,
        activity: &OngoingActivity,
    ) -> Result<SliceOutcome, Box<dyn Error>> {
        let kind = activity.activity();
        let seed = existence
            .organism(self.settings.clock())
            .0
            .random()
            .next_u64();
        let mut configuration = EpisodeConfiguration::new(seed, true, true)
            .with_activity(kind)
            .with_maximum_invocations(Self::SLICE_INVOCATIONS);

        if !kind.is_conscious() || !self.settings.influences() {
            configuration = configuration.without_influences();
        }

        let event = match (kind, activity.focus()) {
            (Activity::Action, ActivityFocus::Event(event)) => event.clone(),
            // Acting on its own concern: a question it gives itself, pursued
            // in reality with all its capabilities.
            (Activity::Action, ActivityFocus::Concern { subject, .. }) => Event::internal(
                format!("initiative-{}", Uuid::now_v7()),
                vec![subject.clone()],
            ),
            (Activity::Introspection, _) => {
                let lived = existence.clock().lived();

                configuration = configuration
                    .perceiving_self(Self::self_observations(existence), format!("{lived:.0}"));

                Event::internal(
                    format!("introspection-{}", Uuid::now_v7()),
                    self.reflective_questions(),
                )
            }
            _ => {
                let questions = self.reflective_questions();
                let draw = existence
                    .organism(self.settings.clock())
                    .0
                    .random()
                    .uniform();
                let pick = ((draw * questions.len() as f64) as usize)
                    .min(questions.len().saturating_sub(1));

                Event::internal(
                    format!("reflection-{}", Uuid::now_v7()),
                    questions.get(pick).cloned().into_iter().collect(),
                )
            }
        };

        let report = {
            let (mut organism, memory) = existence.organism(self.settings.clock());

            self.intelligence
                .reason_with(&event, memory, configuration, false, &mut organism)
        };
        let episode = report.episode().clone();

        self.digest(existence, &episode);

        let answered = !episode.answers().is_empty()
            && episode.answers().iter().all(|answer| {
                matches!(
                    answer.status(),
                    AnswerStatus::Proven | AnswerStatus::Refuted
                )
            });
        let abandoned = episode.termination() == crate::memory::termination::Termination::Abandoned;
        let ending = match kind {
            _ if abandoned => Some(ActivityEnding::Abandoned),
            Activity::Action if answered => {
                // Only a solicitation from outside expects a reply in the
                // outbox; nothing is sent back to another individual
                // automatically.
                if event.is_external() {
                    self.respond(existence, &event, &episode)?;
                }

                Some(ActivityEnding::Completed)
            }
            // An event asking nothing is done with once it has been
            // perceived and reasoned upon.
            Activity::Action if episode.questions().is_empty() => Some(ActivityEnding::Completed),
            Activity::Action => None,
            Activity::Reflection
                if !episode.corrections().is_empty()
                    || !episode.influences_proposed().is_empty()
                    || !episode.experiments().is_empty() =>
            {
                None
            }
            _ => Some(ActivityEnding::Completed),
        };

        let mut follow = None;

        if kind == Activity::Introspection {
            // Attending to its conscious concerns changes their salience.
            let gain = self.channel(existence, Channel::ConcernGain);
            let lived = existence.clock().lived();
            let conscious: Vec<Uuid> = existence
                .concerns()
                .active()
                .filter(|concern| concern.is_conscious())
                .map(|concern| concern.id())
                .collect();

            for id in &conscious {
                if let Some(concern) = existence.concerns_mut().get_mut(*id) {
                    concern.attend(gain.powf(Self::ATTENTION), lived);
                }
            }

            if !conscious.is_empty() {
                self.feel(
                    existence,
                    "revisit:concerns",
                    (1.0 + conscious.len() as f64).ln(),
                );
            }

            // Deliberation: having perceived its concerns, the conscious may
            // take one up, in proportion to its salience.
            let deliberation = self.channel(existence, Channel::Deliberation);
            let saliences: Vec<f64> = conscious
                .iter()
                .map(|id| {
                    existence
                        .concerns()
                        .get(*id)
                        .map_or(0.0, |concern| concern.salience())
                })
                .collect();
            let (draw, chosen) = {
                let mut organism = existence.organism(self.settings.clock()).0;
                let random = organism.random();

                (random.uniform(), random.weighted(&saliences))
            };

            // Taken up in imagination or in reality, in proportion to the
            // current rates of simulating and of acting.
            let (simulate, act) = {
                let organism = existence.organism(self.settings.clock()).0;

                (
                    organism.decision(Channel::Simulate, 0.0).rate(),
                    organism.decision(Channel::Attend, 0.0).rate(),
                )
            };
            let modes: Vec<(Activity, f64)> =
                [(Activity::Simulation, simulate), (Activity::Action, act)]
                    .into_iter()
                    .filter(|(mode, _)| self.settings.allows(*mode))
                    .collect();
            let mode = existence
                .organism(self.settings.clock())
                .0
                .random()
                .weighted(&modes.iter().map(|(_, rate)| *rate).collect::<Vec<_>>())
                .map(|index| modes[index].0);

            if let Some(mode) = mode
                && draw < deliberation
                && let Some(index) = chosen
                && let Some(concern) = existence.concerns().get(conscious[index])
            {
                follow = Some(OngoingActivity::new(
                    mode,
                    ActivityOrigin::Voluntary(activity.id()),
                    ActivityFocus::Concern {
                        id: concern.id(),
                        subject: concern.subject().clone(),
                    },
                    lived,
                ));
            }
        }

        Ok(SliceOutcome {
            summary: format!(
                "{} invocations, {}, {} answers ({} definite), {} corrections, {} influences proposed",
                episode.invocations().len(),
                episode.termination().label(),
                episode.answers().len(),
                episode
                    .answers()
                    .iter()
                    .filter(|answer| matches!(
                        answer.status(),
                        AnswerStatus::Proven | AnswerStatus::Refuted
                    ))
                    .count(),
                episode.corrections().len(),
                episode.influences_proposed().len()
            ),
            ending,
            episode: Some(episode.id()),
            follow,
        })
    }

    /// What an episode leaves among the concerns: the root impasses of an
    /// action form concerns, its definite answers resolve the concerns they answer, and
    /// contradictions found by the examination become concerns too.
    fn digest(&self, existence: &mut Existence, episode: &EpisodeRecord) {
        let answers: Vec<Statement> = episode
            .answers()
            .iter()
            .filter(|answer| matches!(answer.status(), AnswerStatus::Proven))
            .flat_map(|answer| answer.answers().iter().cloned())
            .collect();
        let resolved = existence.concerns_mut().resolve(&answers);

        for concern in &resolved {
            existence.record(LifeEvent::ConcernConcluded {
                concern: *concern,
                how: "resolved".to_owned(),
            });
        }

        if !resolved.is_empty() {
            self.feel(
                existence,
                "concern:resolved",
                (1.0 + resolved.len() as f64).ln(),
            );
        }

        // An examination that finds nothing is no impasse: only the needs
        // met while acting on the world can be stuck.
        for impasse in episode.impasses() {
            if episode.activity() == Activity::Action && impasse.kind().is_root() {
                self.concern(
                    existence,
                    impasse.need().clone(),
                    "impasse".to_owned(),
                    Self::IMPASSE_SALIENCE,
                );
            }
        }

        for answer in &answers {
            if answer.predicate() == "knowledge_concern"
                && answer
                    .arguments()
                    .first()
                    .is_some_and(|kind| kind.to_string() == "contradiction")
            {
                self.concern(
                    existence,
                    answer.clone(),
                    "examination".to_owned(),
                    Self::EXAMINATION_SALIENCE,
                );
            }
        }
    }

    fn respond(
        &self,
        existence: &mut Existence,
        event: &Event,
        episode: &EpisodeRecord,
    ) -> Result<(), Box<dyn Error>> {
        let answers = episode
            .answers()
            .iter()
            .map(|answer| {
                (
                    answer.question().to_string(),
                    answer.status().label().to_owned(),
                    answer.answers().iter().map(ToString::to_string).collect(),
                )
            })
            .collect::<Vec<_>>();
        let count = answers.len();

        self.mailbox.respond(
            event.id(),
            &Response::new(event.id().to_owned(), existence.clock().lived(), answers),
        )?;
        existence.record(LifeEvent::Response {
            event: event.id().to_owned(),
            answers: count,
        });
        self.feel(existence, "respond:event", 1.0);
        Ok(())
    }

    /// A slice of imagination: one unfolding (a dream may take up an
    /// experiment the unconscious asked for).
    fn imagine(&self, existence: &mut Existence, activity: &OngoingActivity) -> SliceOutcome {
        let kind = activity.activity();
        let themes: Vec<Statement> = existence
            .concerns()
            .active()
            .filter(|concern| kind == Activity::Dream || concern.is_conscious())
            .map(|concern| concern.subject().clone())
            .collect();
        let request = (kind == Activity::Dream)
            .then(|| existence.memory_mut().dreams_mut().take(0))
            .flatten();
        let seed = existence
            .organism(self.settings.clock())
            .0
            .random()
            .next_u64();
        let dream = {
            let (mut organism, memory) = existence.organism(self.settings.clock());
            let memory: &crate::memory::persistent_memory::PersistentMemory = memory;

            Imagination::new(self.intelligence, memory, seed)
                .directed(kind, activity.focus().subject())
                .themed(&themes)
                .unfold(request.as_ref(), &mut organism)
        };

        let mechanisms: Vec<String> = dream
            .scenes()
            .iter()
            .flat_map(|scene| scene.mechanisms().to_vec())
            .collect();
        let needs: Vec<Statement> = dream
            .scenes()
            .iter()
            .flat_map(|scene| scene.emerged_needs().to_vec())
            .collect();
        let summary = format!(
            "{} {} ({}): {} scenes [{}], {} needs emerged, {} conjectures{}",
            dream.activity().label(),
            activity.focus().describe(),
            dream.reason(),
            dream.scenes().len(),
            dream
                .scenes()
                .iter()
                .map(|scene| scene.mechanisms().join("+"))
                .collect::<Vec<_>>()
                .join(" | "),
            needs.len(),
            dream.hypotheses().len(),
            dream
                .replays()
                .map(|scene| format!(
                    "; replay {:?}: {}",
                    scene.verdict(),
                    scene.findings().join("; ")
                ))
                .collect::<String>()
        );

        for mechanism in mechanisms {
            self.feel(existence, format!("imagine:{mechanism}"), 1.0);
        }

        for need in needs.into_iter().take(Self::IMAGINED_CONCERNS) {
            self.concern(
                existence,
                need,
                format!("imagination:{}", kind.label()),
                Self::IMAGINATION_SALIENCE,
            );
        }

        if let Some(id) = activity.focus().concern() {
            let gain = self.channel(existence, Channel::ConcernGain);
            let lived = existence.clock().lived();

            if let Some(concern) = existence.concerns_mut().get_mut(id) {
                concern.attend(gain.powf(Self::ATTENTION), lived);
            }
        }

        Dreaming::integrate(existence.memory_mut(), dream);

        SliceOutcome {
            summary,
            ending: Some(ActivityEnding::Completed),
            episode: None,
            follow: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Life;
    use crate::{
        capabilities::{
            dream_interpretation::DreamInterpretation, evidence_review::EvidenceReview,
            hypothesis_formation::HypothesisFormation, learning_review::LearningReview,
            logical_inference::LogicalInference,
        },
        cognition::{module_registry::ModuleRegistry, scripted_module::ScriptedModule},
        engine::{intelligence::Intelligence, learning_parameters::LearningParameters},
        events::event::Event,
        individuality::{
            birth_parameters::BirthParameters, channel::Channel, constitution::Constitution,
        },
        living::{
            clock_mode::ClockMode, existence::Existence, existence_store::ExistenceStore,
            life_event::LifeEvent, life_settings::LifeSettings,
            scheduled_stimulus::ScheduledStimulus,
        },
        memory::termination::Termination,
        resources::initial_resources::InitialResources,
    };

    fn intelligence() -> Intelligence {
        let mut registry = ModuleRegistry::default();

        registry.register(ScriptedModule::new(
            "gatherer",
            Some("item(?x)"),
            "fact(?x, 1)",
        ));
        registry.register(ScriptedModule::new("lister", None, "item(a)"));
        registry.register(LogicalInference::new());
        registry.register(HypothesisFormation::new(3));
        registry.register(EvidenceReview::new());
        registry.register(LearningReview::new());
        registry.register(DreamInterpretation::new());

        Intelligence::new(
            registry,
            InitialResources::default(),
            LearningParameters::default().with_cost_weight_per_second(0.0),
        )
    }

    fn schedule() -> Vec<ScheduledStimulus> {
        ron::de::from_str(
            r#"[
                (at: 60.0, event: (id: "first", source: "test", occurred_at: "2026-11-26T00:00:00Z",
                    payload: assertions(statements: ["context(x)"]), questions: ["fact(?which, 1)"])),
                (at: 7200.0, event: (id: "second", source: "test", occurred_at: "2026-11-26T00:00:00Z",
                    payload: assertions(statements: ["context(y)"]), questions: ["fact(?which, 1)"])),
            ]"#,
        )
        .unwrap()
    }

    fn live(
        existence: &mut Existence,
        hours: f64,
        schedule: Vec<ScheduledStimulus>,
    ) -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        let store = ExistenceStore::open(directory.path()).unwrap();
        let settings = LifeSettings::new(
            ClockMode::Virtual,
            hours * 3_600.0,
            directory.path().to_path_buf(),
        )
        .with_schedule(schedule);

        Life::new(&intelligence(), settings)
            .live(existence, &store)
            .unwrap();
        directory
    }

    fn born(seed: u64) -> Existence {
        Existence::born(format!("i{seed}"), BirthParameters::new(seed, 8).unwrap())
    }

    fn kinds(existence: &Existence) -> Vec<String> {
        existence
            .timeline()
            .entries()
            .iter()
            .filter(|entry| !matches!(entry.event(), LifeEvent::Occurrence { .. }))
            .map(|entry| format!("{:.0} {}", entry.lived(), entry.event().kind()))
            .collect()
    }

    #[test]
    fn the_same_birth_under_the_same_stimuli_lives_the_same_existence() {
        let mut first = born(11);
        let mut second = born(11);
        let mut other = born(12);

        live(&mut first, 6.0, schedule());
        live(&mut second, 6.0, schedule());
        live(&mut other, 6.0, schedule());

        assert_eq!(kinds(&first), kinds(&second));
        assert_eq!(first.latent(), second.latent());
        assert_ne!(kinds(&first), kinds(&other));
    }

    #[test]
    fn may_answer_or_neglect_a_stimulus_according_to_its_laws() {
        let mut attentive = born(21);
        let mut inattentive = born(21);

        attentive.constitution_mut().set_bias(Channel::Attend, 20.0);
        inattentive
            .constitution_mut()
            .set_bias(Channel::Attend, -40.0);
        inattentive
            .constitution_mut()
            .set_bias(Channel::FallAsleep, -40.0);
        inattentive
            .constitution_mut()
            .set_bias(Channel::ConcernFading, 3.0);

        let directory = live(&mut attentive, 1.0, schedule());

        assert!(directory.path().join("outbox/first.ron").is_file());

        let directory = live(&mut inattentive, 12.0, schedule());

        // It had every capability to answer, and did not.
        assert!(!directory.path().join("outbox/first.ron").exists());
        assert!(
            inattentive
                .timeline()
                .entries()
                .iter()
                .any(|entry| matches!(entry.event(), LifeEvent::StimulusNeglected { .. }))
        );
        assert!(
            inattentive
                .concerns()
                .concerns()
                .iter()
                .any(|concern| concern.origin().starts_with("neglected"))
        );
    }

    #[test]
    fn the_internal_state_can_make_the_reasoner_give_up() {
        let intelligence = intelligence();
        let event: Event = ron::de::from_str(
            r#"(id: "e", source: "test", occurred_at: "2026-11-26T00:00:00Z",
                payload: assertions(statements: ["context(x)"]), questions: ["fact(?which, 2)"])"#,
        )
        .unwrap();
        let configuration = || {
            crate::memory::episode_configuration::EpisodeConfiguration::new(1, true, true)
                .with_maximum_invocations(30)
        };
        let mut existence = born(31);

        existence
            .constitution_mut()
            .set_bias(Channel::Abandonment, 20.0);

        let report = {
            let (mut organism, memory) = existence.organism(ClockMode::Virtual);

            intelligence.reason_with(&event, memory, configuration(), false, &mut organism)
        };

        assert_eq!(report.episode().termination(), Termination::Abandoned);
        assert!(existence.timeline().totals()["occurrence"] > 0);

        // Without internal state, the same reasoning goes on.
        let neutral = intelligence.reason(&event, existence.memory_mut(), configuration(), false);

        assert_ne!(neutral.episode().termination(), Termination::Abandoned);
    }

    #[test]
    fn resumes_its_existence_after_a_technical_stop() {
        let mut existence = born(41);
        let directory = live(&mut existence, 2.0, schedule());
        let store = ExistenceStore::open(directory.path()).unwrap();
        let mut restored = store.load().unwrap();

        assert_eq!(restored.clock().lived(), existence.clock().lived());
        assert_eq!(restored.latent(), existence.latent());
        drop(store);

        // A long stop relaxes the state, without anything being lived.
        let lived = restored.clock().lived();
        let store = ExistenceStore::open(directory.path()).unwrap();
        let settings = LifeSettings::new(ClockMode::Virtual, 0.0, directory.path().to_path_buf())
            .with_dormancy(1e7);

        Life::new(&intelligence(), settings)
            .live(&mut restored, &store)
            .unwrap();

        assert_eq!(restored.clock().lived(), lived);
        assert_eq!(restored.clock().sessions(), 2);
        assert_eq!(restored.clock().dormant(), 1e7);
        assert_ne!(restored.latent(), existence.latent());
    }

    #[test]
    fn extreme_individuals_stay_technically_valid() {
        for (scale, factor) in [
            ("coupling_gain", 20.0),
            ("noise", 50.0),
            ("impulse_scale", 100.0),
            ("sensitivity_scale", 30.0),
            ("gain_plasticity", 1_000.0),
            ("coupling_plasticity", 1_000.0),
            ("baseline_plasticity", 1_000.0),
            ("habit_plasticity", 1_000.0),
            ("time_scale", 1e-6),
            ("bias_spread", 25.0),
        ] {
            let birth = BirthParameters::new(51, 8).unwrap();
            let scales = Constitution::generate(birth.clone())
                .scales()
                .clone()
                .scaled(scale, factor)
                .unwrap();
            let mut existence = born(51);

            *existence.constitution_mut() = Constitution::from_scales(birth, scales);
            live(&mut existence, 3.0, schedule());

            assert!(
                existence
                    .latent()
                    .pre()
                    .iter()
                    .all(|value| value.is_finite()),
                "{scale}"
            );
            assert_eq!(existence.latent().repairs(), 0, "{scale}");
            assert!(existence.clock().lived() >= 3.0 * 3_600.0, "{scale}");
        }
    }
}
