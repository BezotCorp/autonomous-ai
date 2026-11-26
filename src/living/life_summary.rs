use std::{collections::BTreeMap, fmt};

use serde::Serialize;

use crate::{
    individuality::channel::Channel,
    living::{
        concern_status::ConcernStatus, existence::Existence, life_event::LifeEvent,
        vigilance::Vigilance,
    },
};

/// Observed summary of an existence, computed from what was recorded. It
/// reports counts and measures; it names no psychological state and
/// interprets nothing.
#[derive(Debug, Serialize)]
pub(crate) struct LifeSummary {
    name: String,
    seed: u64,
    dimensions: usize,
    lived_seconds: f64,
    dormant_seconds: f64,
    sessions: u64,
    vigilance: Vigilance,
    asleep_share: f64,
    /// Per kind of activity: started, by origin; ended, by ending; lived seconds.
    activities: BTreeMap<String, ActivityCounts>,
    ongoing: usize,
    suspended: usize,
    ended_not_kept: u64,
    stimuli_received: u64,
    stimuli_neglected: u64,
    stimuli_waiting: usize,
    responses: u64,
    concerns_formed: u64,
    concerns_resolved: u64,
    concerns_faded: u64,
    concerns_active: usize,
    concerns_conscious: usize,
    most_activated_concerns: Vec<(String, String, u64, f64)>,
    occurrences: u64,
    distinct_occurrences: usize,
    latent_mean: Vec<f64>,
    latent_deviation: Vec<f64>,
    latent_path: f64,
    channels: Vec<(String, f64, f64, f64)>,
    coupling_change: f64,
    baseline_change: f64,
    habits: usize,
    engagements: u64,
    plastic_hours: f64,
    extreme_gains: Vec<(String, f64)>,
    repairs: u64,
    technical_errors: u64,
    interventions: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
struct ActivityCounts {
    started: BTreeMap<String, u64>,
    ended: BTreeMap<String, u64>,
    lived_seconds: f64,
    slices: u64,
}

impl LifeSummary {
    pub(crate) fn of(existence: &Existence) -> Self {
        let identity = existence.identity();
        let timeline = existence.timeline();
        let mut activities: BTreeMap<String, ActivityCounts> = BTreeMap::new();
        let mut samples: Vec<&[f64]> = Vec::new();
        let mut asleep = 0.0;
        let mut since: Option<f64> = None;
        let mut first = None;
        let mut interventions = Vec::new();

        for entry in timeline.entries() {
            first.get_or_insert(entry.lived());

            match entry.event() {
                LifeEvent::ActivityStarted { kind, origin, .. } => {
                    *activities
                        .entry(kind.label().to_owned())
                        .or_default()
                        .started
                        .entry(origin.label().to_owned())
                        .or_default() += 1;
                }
                LifeEvent::Latent { state } => samples.push(state),
                LifeEvent::Vigilance { to, .. } => match to {
                    Vigilance::Asleep => since = Some(entry.lived()),
                    Vigilance::Awake => {
                        if let Some(start) = since.take() {
                            asleep += entry.lived() - start;
                        }
                    }
                },
                LifeEvent::Intervention { description } => interventions.push(description.clone()),
                _ => {}
            }
        }

        let lived = existence.clock().lived();

        if let Some(start) = since {
            asleep += lived - start;
        }

        for ended in existence.activities().ended() {
            let counts = activities
                .entry(ended.activity().label().to_owned())
                .or_default();

            *counts
                .ended
                .entry(ended.ending().label().to_owned())
                .or_default() += 1;
            counts.lived_seconds += ended.lived();
            counts.slices += u64::from(ended.slices());
        }

        let dimensions = existence.constitution().dimensions();
        let count = samples.len().max(1) as f64;
        let latent_mean: Vec<f64> = (0..dimensions)
            .map(|index| samples.iter().map(|state| state[index]).sum::<f64>() / count)
            .collect();
        let latent_deviation = (0..dimensions)
            .map(|index| {
                (samples
                    .iter()
                    .map(|state| (state[index] - latent_mean[index]).powi(2))
                    .sum::<f64>()
                    / count)
                    .sqrt()
            })
            .collect();
        let latent_path = samples
            .windows(2)
            .map(|pair| {
                pair[0]
                    .iter()
                    .zip(pair[1])
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt()
            })
            .sum();

        let concerns = existence.concerns();
        let mut most_activated: Vec<_> = concerns
            .concerns()
            .iter()
            .map(|concern| {
                (
                    concern.subject().to_string(),
                    concern.origin().to_owned(),
                    concern.activations(),
                    concern.salience(),
                )
            })
            .collect();

        most_activated.sort_by_key(|concern| std::cmp::Reverse(concern.2));
        most_activated.truncate(8);

        let dispositions = existence.dispositions();
        let constitution = existence.constitution();
        let latent = existence.latent().activation();
        let channels = Channel::ALL
            .iter()
            .map(|channel| {
                let (bias, term) = dispositions.activation(constitution, *channel, &latent);

                (
                    channel.label(),
                    bias,
                    term,
                    crate::individuality::channel_law::ChannelLaw::shape(
                        channel.form(),
                        bias + term,
                    ),
                )
            })
            .collect();
        let mut gains: Vec<(String, f64)> = dispositions
            .gains()
            .iter()
            .map(|(signature, gain)| (signature.clone(), gain.exp()))
            .collect();

        gains.sort_by(|left, right| (right.1.ln().abs()).total_cmp(&left.1.ln().abs()));
        gains.truncate(8);

        let totals = timeline.totals();
        let total = |kind: &str| totals.get(kind).copied().unwrap_or(0);
        let concluded = |status: ConcernStatus| {
            concerns
                .concerns()
                .iter()
                .filter(|concern| concern.status() == status)
                .count() as u64
        };

        Self {
            name: identity.name().to_owned(),
            seed: identity.birth().seed(),
            dimensions,
            lived_seconds: lived,
            dormant_seconds: existence.clock().dormant(),
            sessions: existence.clock().sessions(),
            vigilance: existence.vigilance(),
            asleep_share: if lived > first.unwrap_or(0.0) {
                asleep / (lived - first.unwrap_or(0.0))
            } else {
                0.0
            },
            activities,
            ongoing: existence.activities().running().count(),
            suspended: existence.activities().suspended().count(),
            ended_not_kept: existence.activities().dropped(),
            stimuli_received: total("stimulus_received"),
            stimuli_neglected: total("stimulus_neglected"),
            stimuli_waiting: existence.stimuli().len(),
            responses: total("response"),
            concerns_formed: total("concern_formed"),
            concerns_resolved: concluded(ConcernStatus::Resolved),
            concerns_faded: concluded(ConcernStatus::Faded),
            concerns_active: concerns.active().count(),
            concerns_conscious: concerns
                .active()
                .filter(|concern| concern.is_conscious())
                .count(),
            most_activated_concerns: most_activated,
            occurrences: total("occurrence"),
            distinct_occurrences: existence.catalog().reactions().len(),
            latent_mean,
            latent_deviation,
            latent_path,
            channels,
            coupling_change: dispositions.coupling_change().frobenius()
                / constitution.coupling().frobenius().max(f64::MIN_POSITIVE),
            baseline_change: dispositions
                .baseline_change()
                .iter()
                .map(|value| value * value)
                .sum::<f64>()
                .sqrt(),
            habits: dispositions.sensitivity_changes().len(),
            engagements: dispositions.engagements(),
            plastic_hours: dispositions.consolidated_hours(),
            extreme_gains: gains,
            repairs: existence.latent().repairs(),
            technical_errors: total("technical_error"),
            interventions,
        }
    }
}

impl fmt::Display for LifeSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vector = |values: &[f64]| {
            values
                .iter()
                .map(|value| format!("{value:+.2}"))
                .collect::<Vec<_>>()
                .join(" ")
        };

        writeln!(
            formatter,
            "{} (seed {}, {} dimensions): lived {:.0} s over {} sessions, dormant {:.0} s, now {}, asleep {:.0}% of the recorded time",
            self.name,
            self.seed,
            self.dimensions,
            self.lived_seconds,
            self.sessions,
            self.dormant_seconds,
            self.vigilance.label(),
            100.0 * self.asleep_share
        )?;
        writeln!(formatter, "activities:")?;

        for (kind, counts) in &self.activities {
            writeln!(
                formatter,
                "  {kind:<13} started {:?}  ended {:?}  {} slices, {:.0} s",
                counts.started, counts.ended, counts.slices, counts.lived_seconds
            )?;
        }

        writeln!(
            formatter,
            "  under way: {} running, {} suspended ({} old finished activities no longer kept in detail)",
            self.ongoing, self.suspended, self.ended_not_kept
        )?;
        writeln!(
            formatter,
            "stimuli: {} received, {} responses, {} neglected, {} waiting",
            self.stimuli_received, self.responses, self.stimuli_neglected, self.stimuli_waiting
        )?;
        writeln!(
            formatter,
            "concerns: {} formed, {} resolved, {} faded, {} active ({} conscious)",
            self.concerns_formed,
            self.concerns_resolved,
            self.concerns_faded,
            self.concerns_active,
            self.concerns_conscious
        )?;

        for (subject, origin, activations, salience) in &self.most_activated_concerns {
            writeln!(
                formatter,
                "  {activations:>4} activations  salience {salience:.2}  {subject}  ({origin})"
            )?;
        }

        writeln!(
            formatter,
            "occurrences: {} ({} distinct signatures)",
            self.occurrences, self.distinct_occurrences
        )?;
        writeln!(formatter, "latent mean      {}", vector(&self.latent_mean))?;
        writeln!(
            formatter,
            "latent deviation {}",
            vector(&self.latent_deviation)
        )?;
        writeln!(
            formatter,
            "latent path length {:.2} (over recorded samples)",
            self.latent_path
        )?;
        writeln!(formatter, "channels now (bias, latent term, value):")?;

        for (channel, bias, term, value) in &self.channels {
            writeln!(
                formatter,
                "  {channel:<20} {bias:+7.2} {term:+7.2}  {value:.4}"
            )?;
        }

        writeln!(
            formatter,
            "plasticity (over {:.2} lived hours): coupling changed by {:.1}% of its born norm, resting point moved by {:.3}, {} habits over {} engagements",
            self.plastic_hours,
            100.0 * self.coupling_change,
            self.baseline_change,
            self.habits,
            self.engagements
        )?;

        for (signature, gain) in &self.extreme_gains {
            writeln!(formatter, "  gain {gain:.3}  {signature}")?;
        }

        writeln!(
            formatter,
            "technical: {} latent repairs, {} technical errors",
            self.repairs, self.technical_errors
        )?;

        for intervention in &self.interventions {
            writeln!(formatter, "intervention: {intervention}")?;
        }

        Ok(())
    }
}
