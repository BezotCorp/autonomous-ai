use std::collections::BTreeMap;

use serde::Serialize;

use crate::memory::{dream_record::DreamRecord, dream_verdict::DreamVerdict};

/// What dreams were made of and what their replays found: scenes,
/// mechanisms and transitions used, needs that emerged, capabilities
/// imagined, and, for replays (historical and reinterpretations apart),
/// verdicts and simulated gains. Counting them proves no improvement: they
/// describe the activity, whose effects are measured on the conscious.
#[derive(Debug, Default, Serialize)]
pub(crate) struct DreamStatistics {
    dreams: usize,
    scenes: usize,
    seconds: f64,
    mechanisms: BTreeMap<String, usize>,
    transitions: BTreeMap<String, usize>,
    emerged_needs: usize,
    imagined_capabilities: usize,
    imagined_consequences: usize,
    conjectures: usize,
    questions_raised: usize,
    strategies_explored: usize,
    scene_seconds: f64,
    replay_verdicts: BTreeMap<String, usize>,
    simulated_invocations_saved: i64,
    simulated_consumption_saved: f64,
}

impl DreamStatistics {
    pub(crate) fn observe(&mut self, dream: &DreamRecord) {
        self.dreams += 1;
        self.seconds += dream.seconds();
        self.conjectures += dream.hypotheses().len();

        for scene in dream.scenes() {
            self.scenes += 1;
            self.emerged_needs += scene.emerged_needs().len();
            self.imagined_capabilities += scene.capabilities().len();
            self.imagined_consequences += scene.consequences().len();
            self.questions_raised += scene.questions().len();
            self.strategies_explored += scene.strategies().len();
            self.scene_seconds += scene.seconds();
            *self
                .transitions
                .entry(scene.transition().to_owned())
                .or_default() += 1;

            for mechanism in scene.mechanisms() {
                let mechanism = mechanism.split(':').next().unwrap_or_default().to_owned();

                *self.mechanisms.entry(mechanism).or_default() += 1;
            }

            if let Some((before, after, verdict)) = scene.comparison() {
                *self
                    .replay_verdicts
                    .entry(format!("{:?} {verdict:?}", scene.mode()))
                    .or_default() += 1;

                if *verdict == DreamVerdict::Cheaper {
                    self.simulated_invocations_saved +=
                        before.invocations() as i64 - after.invocations() as i64;
                    self.simulated_consumption_saved +=
                        before.total_consumption() - after.total_consumption();
                }
            }
        }
    }
}
