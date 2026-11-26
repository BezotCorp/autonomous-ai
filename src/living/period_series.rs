use std::fmt;

use serde::Serialize;

use crate::living::{existence::Existence, life_event::LifeEvent};

/// Consecutive periods of equal length of lived time, one line each, read
/// in a single pass over the timeline: the evolution of an existence
/// without printing its history.
#[derive(Debug, Serialize)]
pub(crate) struct PeriodSeries {
    every: f64,
    periods: Vec<PeriodLine>,
}

#[derive(Debug, Default, Serialize)]
struct PeriodLine {
    start: f64,
    started: u64,
    abandoned: u64,
    interrupted: u64,
    stimuli: u64,
    neglected: u64,
    transmitted: u64,
    received_from_individuals: u64,
    concerns: u64,
    occurrences: u64,
    asleep_changes: u64,
    latent_path: f64,
}

impl PeriodSeries {
    /// Bound on the lines of one series.
    const MAXIMUM_PERIODS: usize = 100_000;

    pub(crate) fn of(
        existence: &Existence,
        from: f64,
        to: f64,
        every: f64,
    ) -> Result<Self, String> {
        if !(every.is_finite() && every > 0.0) {
            return Err("the period length must be positive".to_owned());
        }

        let entries = existence.timeline().entries();
        let first = entries.front().map_or(0.0, |entry| entry.lived()).max(from);
        let last = entries.back().map_or(0.0, |entry| entry.lived()).min(to);

        if last < first {
            return Ok(Self {
                every,
                periods: Vec::new(),
            });
        }

        let count = ((last - first) / every).floor() as usize + 1;

        if count > Self::MAXIMUM_PERIODS {
            return Err(format!(
                "{count} periods: choose a longer period or a shorter interval"
            ));
        }

        let mut periods: Vec<PeriodLine> = (0..count)
            .map(|index| PeriodLine {
                start: first + index as f64 * every,
                ..PeriodLine::default()
            })
            .collect();
        let mut previous: Option<&[f64]> = None;

        for entry in entries {
            if entry.lived() < first || entry.lived() > last {
                continue;
            }

            let index = (((entry.lived() - first) / every) as usize).min(count - 1);
            let line = &mut periods[index];

            match entry.event() {
                LifeEvent::ActivityStarted { .. } => line.started += 1,
                LifeEvent::ActivityEnded { ending, .. } => match ending.label() {
                    "abandoned" => line.abandoned += 1,
                    "interrupted" => line.interrupted += 1,
                    _ => {}
                },
                LifeEvent::StimulusReceived { .. } => line.stimuli += 1,
                LifeEvent::StimulusNeglected { .. } => line.neglected += 1,
                LifeEvent::Transmitted { .. } => line.transmitted += 1,
                LifeEvent::ConcernFormed { .. } => line.concerns += 1,
                LifeEvent::Vigilance { .. } => line.asleep_changes += 1,
                LifeEvent::Occurrence { signature, .. } => {
                    line.occurrences += 1;

                    if signature.starts_with("stimulus:individual:") {
                        line.received_from_individuals += 1;
                    }
                }
                LifeEvent::Latent { state } => {
                    if let Some(previous) = previous {
                        line.latent_path += previous
                            .iter()
                            .zip(state)
                            .map(|(a, b)| (a - b).powi(2))
                            .sum::<f64>()
                            .sqrt();
                    }

                    previous = Some(state);
                }
                _ => {}
            }
        }

        Ok(Self { every, periods })
    }
}

impl fmt::Display for PeriodSeries {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "start,started,abandoned,interrupted,stimuli,neglected,transmitted,received_from_individuals,concerns,occurrences,vigilance_changes,latent_path"
        )?;

        for line in &self.periods {
            writeln!(
                formatter,
                "{:.0},{},{},{},{},{},{},{},{},{},{},{:.3}",
                line.start,
                line.started,
                line.abandoned,
                line.interrupted,
                line.stimuli,
                line.neglected,
                line.transmitted,
                line.received_from_individuals,
                line.concerns,
                line.occurrences,
                line.asleep_changes,
                line.latent_path
            )?;
        }

        Ok(())
    }
}
