use std::{collections::BTreeSet, path::PathBuf};

use crate::{
    living::{clock_mode::ClockMode, scheduled_stimulus::ScheduledStimulus},
    memory::activity::Activity,
};

/// Conditions of one living session. They belong to the experiment, not
/// to the individual: nothing here is persisted in the existence, except
/// as an intervention recorded in its timeline when it departs from the
/// normal conditions.
#[derive(Clone, Debug)]
pub(crate) struct LifeSettings {
    clock: ClockMode,
    duration: f64,
    checkpoint: f64,
    directory: PathBuf,
    schedule: Vec<ScheduledStimulus>,
    excluded: BTreeSet<Activity>,
    influences: bool,
    dormancy: Option<f64>,
}

impl LifeSettings {
    pub(crate) fn new(clock: ClockMode, duration: f64, directory: PathBuf) -> Self {
        Self {
            clock,
            duration,
            checkpoint: 60.0,
            directory,
            schedule: Vec::new(),
            excluded: BTreeSet::new(),
            influences: true,
            dormancy: None,
        }
    }

    /// The same conditions for the existence living in another directory.
    pub(crate) fn in_directory(mut self, directory: PathBuf) -> Self {
        self.directory = directory;
        self
    }

    /// Wall-clock seconds between two saves.
    pub(crate) fn with_checkpoint(mut self, seconds: f64) -> Self {
        self.checkpoint = seconds;
        self
    }

    pub(crate) fn with_schedule(mut self, mut schedule: Vec<ScheduledStimulus>) -> Self {
        schedule.sort_by(|left, right| left.at().total_cmp(&right.at()));
        self.schedule = schedule;
        self
    }

    /// Kinds of activity made impossible for the session (experimental
    /// ablation).
    pub(crate) fn without(mut self, excluded: BTreeSet<Activity>) -> Self {
        self.excluded = excluded;
        self
    }

    /// The conscious ignores the influences of the unconscious
    /// (experimental condition).
    pub(crate) fn without_influences(mut self) -> Self {
        self.influences = false;
        self
    }

    /// Dormancy to apply at the opening of the session instead of the
    /// measured wall-clock gap (by default, the gap is applied with a real
    /// clock and ignored with a virtual one).
    pub(crate) fn with_dormancy(mut self, seconds: f64) -> Self {
        self.dormancy = Some(seconds);
        self
    }

    pub(crate) fn clock(&self) -> ClockMode {
        self.clock
    }

    pub(crate) fn duration(&self) -> f64 {
        self.duration
    }

    pub(crate) fn checkpoint(&self) -> f64 {
        self.checkpoint
    }

    pub(crate) fn directory(&self) -> &PathBuf {
        &self.directory
    }

    pub(crate) fn schedule(&self) -> &[ScheduledStimulus] {
        &self.schedule
    }

    pub(crate) fn allows(&self, activity: Activity) -> bool {
        !self.excluded.contains(&activity)
    }

    pub(crate) fn excluded(&self) -> &BTreeSet<Activity> {
        &self.excluded
    }

    pub(crate) fn influences(&self) -> bool {
        self.influences
    }

    pub(crate) fn dormancy(&self, wall_gap: f64) -> f64 {
        match (self.dormancy, self.clock) {
            (Some(seconds), _) => seconds,
            (None, ClockMode::Real) => wall_gap,
            (None, ClockMode::Virtual) => 0.0,
        }
    }
}
