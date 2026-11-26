use std::{collections::BTreeMap, fmt};

use serde::Serialize;

use crate::{
    individuality::channel::Channel,
    living::{existence::Existence, life_event::LifeEvent},
};

/// Coupling change, resting point change and engagements at a sample.
type PlasticSample = (f64, f64, u64);

/// What happened to an existence between two lived times, from its
/// timeline and its memory: counts, differences between the first and the
/// last samples of the period, and what was learned. Observations only.
#[derive(Debug, Serialize)]
pub(crate) struct PeriodReport {
    from: f64,
    to: f64,
    events: BTreeMap<String, u64>,
    started: BTreeMap<String, u64>,
    ended: BTreeMap<String, u64>,
    stimuli: BTreeMap<String, u64>,
    transmitted: BTreeMap<String, u64>,
    concerns: BTreeMap<String, u64>,
    occurrences: Vec<(String, u64)>,
    channels: Vec<(String, f64, f64)>,
    plasticity: Option<(PlasticSample, PlasticSample)>,
    latent_path: f64,
    knowledge: BTreeMap<String, u64>,
    new_statements: Vec<String>,
    /// Observations of the environment that perceived someone.
    perceptions: u64,
    /// Messages received from other individuals (relays included).
    received_from_individuals: u64,
    /// Imagined scenes in which perceiving or transmitting was a question,
    /// never acted upon there: envisaged interactions.
    envisaged: u64,
}

impl PeriodReport {
    const OCCURRENCES: usize = 12;
    const STATEMENTS: usize = 20;
    /// Predicates of the capabilities of perceiving and transmitting.
    const INTERACTIONS: [&'static str; 2] = ["present", "transmitted"];

    pub(crate) fn of(existence: &Existence, from: f64, to: f64) -> Self {
        let mut report = Self {
            from,
            to,
            events: BTreeMap::new(),
            started: BTreeMap::new(),
            ended: BTreeMap::new(),
            stimuli: BTreeMap::new(),
            transmitted: BTreeMap::new(),
            concerns: BTreeMap::new(),
            occurrences: Vec::new(),
            channels: Vec::new(),
            plasticity: None,
            latent_path: 0.0,
            knowledge: BTreeMap::new(),
            new_statements: Vec::new(),
            perceptions: 0,
            received_from_individuals: 0,
            envisaged: 0,
        };
        let mut occurrences: BTreeMap<String, u64> = BTreeMap::new();
        let mut first_sample: Option<(&[f64], f64, f64, u64)> = None;
        let mut last_sample: Option<(&[f64], f64, f64, u64)> = None;
        let mut previous_state: Option<&[f64]> = None;
        let mut wall = (None, None);

        for entry in existence.timeline().entries() {
            if entry.lived() < from || entry.lived() > to {
                continue;
            }

            wall.0.get_or_insert(entry.wall());
            wall.1 = Some(entry.wall());
            *report
                .events
                .entry(entry.event().kind().to_owned())
                .or_default() += 1;

            match entry.event() {
                LifeEvent::ActivityStarted { kind, origin, .. } => {
                    *report
                        .started
                        .entry(format!("{} ({})", kind.label(), origin.label()))
                        .or_default() += 1;
                }
                LifeEvent::ActivityEnded { kind, ending, .. } => {
                    *report
                        .ended
                        .entry(format!("{} ({})", kind.label(), ending.label()))
                        .or_default() += 1;
                }
                LifeEvent::StimulusReceived { event, .. } => {
                    *report
                        .stimuli
                        .entry(if event.contains("/relayed/") {
                            "relayed".to_owned()
                        } else {
                            "received".to_owned()
                        })
                        .or_default() += 1;
                }
                LifeEvent::StimulusNeglected { .. } => {
                    *report.stimuli.entry("neglected".to_owned()).or_default() += 1;
                }
                LifeEvent::Transmitted { to, status, .. } => {
                    *report
                        .transmitted
                        .entry(format!(
                            "{to}: {}",
                            status.split([' ', '{']).next().unwrap_or(status)
                        ))
                        .or_default() += 1;
                }
                LifeEvent::ConcernFormed { origin, .. } => {
                    *report
                        .concerns
                        .entry(origin.split(':').next().unwrap_or(origin).to_owned())
                        .or_default() += 1;
                }
                LifeEvent::Occurrence { signature, .. } => {
                    *occurrences.entry(signature.clone()).or_default() += 1;

                    if signature == "produce:environment_observation" {
                        report.perceptions += 1;
                    }

                    if signature.starts_with("stimulus:individual:") {
                        report.received_from_individuals += 1;
                    }
                }
                LifeEvent::Latent { state } => {
                    if let Some(previous) = previous_state {
                        report.latent_path += previous
                            .iter()
                            .zip(state)
                            .map(|(a, b)| (a - b).powi(2))
                            .sum::<f64>()
                            .sqrt();
                    }

                    previous_state = Some(state);
                }
                LifeEvent::Plasticity {
                    channels,
                    coupling_change,
                    baseline_change,
                    engagements,
                } => {
                    let sample = (
                        channels.as_slice(),
                        *coupling_change,
                        *baseline_change,
                        *engagements,
                    );

                    first_sample.get_or_insert(sample);
                    last_sample = Some(sample);
                }
                _ => {}
            }
        }

        let mut occurrences: Vec<(String, u64)> = occurrences.into_iter().collect();

        occurrences.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        occurrences.truncate(Self::OCCURRENCES);
        report.occurrences = occurrences;

        if let (Some(first), Some(last)) = (first_sample, last_sample) {
            report.channels = Channel::ALL
                .iter()
                .zip(first.0.iter().zip(last.0))
                .map(|(channel, (before, after))| (channel.label(), *before, *after))
                .collect();
            report.plasticity = Some(((first.1, first.2, first.3), (last.1, last.2, last.3)));
        }

        if let (Some(start), Some(end)) = wall {
            for dream in existence.memory().dreams().dreams() {
                if dream.started_at() < start || dream.started_at() > end {
                    continue;
                }

                report.envisaged += dream
                    .scenes()
                    .iter()
                    .filter(|scene| {
                        scene
                            .questions()
                            .iter()
                            .chain(scene.emerged_needs())
                            .any(|need| Self::INTERACTIONS.contains(&need.predicate()))
                    })
                    .count() as u64;
            }

            for entry in existence.memory().knowledge().entries() {
                let first = entry
                    .evidence()
                    .iter()
                    .map(|evidence| evidence.recorded_at())
                    .min();

                if first.is_some_and(|first| first >= start && first <= end) {
                    *report
                        .knowledge
                        .entry(
                            format!("{:?}", entry.standing())
                                .split([' ', '{'])
                                .next()
                                .unwrap_or("")
                                .to_owned(),
                        )
                        .or_default() += 1;

                    if report.new_statements.len() < Self::STATEMENTS {
                        report.new_statements.push(entry.statement().to_string());
                    }
                }
            }
        }

        report
    }
}

impl fmt::Display for PeriodReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "period [{:.0}, {:.0}] s of lived time",
            self.from, self.to
        )?;
        writeln!(formatter, "timeline events: {:?}", self.events)?;
        writeln!(formatter, "activities started: {:?}", self.started)?;
        writeln!(formatter, "activities ended: {:?}", self.ended)?;
        writeln!(formatter, "stimuli: {:?}", self.stimuli)?;
        writeln!(formatter, "transmitted: {:?}", self.transmitted)?;
        writeln!(
            formatter,
            "environment: {} perceptions of someone, {} messages received from individuals, {} imagined scenes envisaging an interaction",
            self.perceptions, self.received_from_individuals, self.envisaged
        )?;
        writeln!(formatter, "concerns formed by origin: {:?}", self.concerns)?;
        writeln!(
            formatter,
            "most frequent occurrences: {:?}",
            self.occurrences
        )?;
        writeln!(formatter, "latent path length: {:.2}", self.latent_path)?;

        match &self.plasticity {
            Some((
                (coupling, baseline, engagements),
                (coupling_after, baseline_after, engagements_after),
            )) => {
                writeln!(
                    formatter,
                    "plasticity: coupling change {coupling:.4} -> {coupling_after:.4}, resting point change {baseline:.4} -> {baseline_after:.4}, engagements {engagements} -> {engagements_after}"
                )?;
            }
            None => writeln!(formatter, "plasticity: no sample in the period")?,
        }

        for (channel, before, after) in &self.channels {
            writeln!(formatter, "  {channel:<20} {before:>12.4} -> {after:>12.4}")?;
        }

        writeln!(
            formatter,
            "knowledge first recorded in the period: {:?}",
            self.knowledge
        )?;

        for statement in &self.new_statements {
            writeln!(formatter, "  {statement}")?;
        }

        Ok(())
    }
}
