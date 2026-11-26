use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    individuality::{channel::Channel, constitution::Constitution, reaction_law::ReactionLaw},
    mathematics::square_matrix::SquareMatrix,
};

/// What the existence has changed in the laws of the individual
/// (plasticity), kept apart from the constitution it was born with, so
/// that both remain inspectable. The effective laws are the born ones plus
/// these changes. Four slow transformations, each with its own individual
/// rate, none of them aimed at any improvement:
///
/// - **reaction gains**: each occurrence changes the gain of its own law
///   by `rate · drift · z`: whether repetition strengthens (sensitisation)
///   or weakens (habituation) the effect depends on the state the
///   individual is in when it happens;
/// - **internal coupling**: dimensions active together change their
///   coupling by `polarity · rate · x_i · x_j` per hour (co-activity rule,
///   reinforcing or inhibiting according to the individual's polarity);
/// - **resting point**: drifts towards the states actually lived, at a
///   slow individual rate;
/// - **habits**: when a mechanism is engaged, the sensitivity of its
///   channel moves towards the change of state that followed the
///   engagement, so that the states an activity leads to come to favour
///   (or disfavour) it, whatever their value for any task.
///
/// Every change is bounded, so that plasticity can drive an individual to
/// extreme but never invalid laws.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Dispositions {
    coupling: SquareMatrix,
    baseline: Vec<f64>,
    sensitivities: BTreeMap<Channel, Vec<f64>>,
    gains: BTreeMap<String, f64>,
    engagements: u64,
    consolidated_hours: f64,
}

impl Dispositions {
    /// Bounds on the logarithm of a reaction gain.
    const GAIN_BOUNDS: (f64, f64) = (-4.0, 3.0);
    /// Bound on the absolute value of an effective resting point.
    const BASELINE_BOUND: f64 = 3.0;
    /// Bound on the change of coupling, relative to the born coupling.
    const COUPLING_BOUND: f64 = 2.0;
    /// Bound on the change of a sensitivity, in units of the individual's
    /// sensitivity scale.
    const SENSITIVITY_BOUND: f64 = 3.0;

    pub(crate) fn new(constitution: &Constitution) -> Self {
        let dimensions = constitution.dimensions();

        Self {
            coupling: SquareMatrix::zero(dimensions),
            baseline: vec![0.0; dimensions],
            sensitivities: BTreeMap::new(),
            gains: BTreeMap::new(),
            engagements: 0,
            consolidated_hours: 0.0,
        }
    }

    pub(crate) fn baseline(&self, constitution: &Constitution) -> Vec<f64> {
        constitution
            .baseline()
            .iter()
            .zip(&self.baseline)
            .map(|(born, change)| born + change)
            .collect()
    }

    pub(crate) fn coupling(&self, constitution: &Constitution) -> SquareMatrix {
        constitution.coupling().plus(&self.coupling)
    }

    pub(crate) fn coupling_change(&self) -> &SquareMatrix {
        &self.coupling
    }

    pub(crate) fn baseline_change(&self) -> &[f64] {
        &self.baseline
    }

    pub(crate) fn sensitivity_change(&self, channel: Channel) -> Option<&[f64]> {
        self.sensitivities.get(&channel).map(Vec::as_slice)
    }

    pub(crate) fn sensitivity_changes(&self) -> &BTreeMap<Channel, Vec<f64>> {
        &self.sensitivities
    }

    pub(crate) fn gains(&self) -> &BTreeMap<String, f64> {
        &self.gains
    }

    pub(crate) fn engagements(&self) -> u64 {
        self.engagements
    }

    pub(crate) fn consolidated_hours(&self) -> f64 {
        self.consolidated_hours
    }

    /// Current gain of the reaction law of an occurrence.
    pub(crate) fn gain(&self, signature: &str) -> f64 {
        self.gains.get(signature).copied().unwrap_or(0.0).exp()
    }

    /// Activation of a channel: born bias, plus the projection of the
    /// latent state through the born and acquired sensitivities.
    pub(crate) fn activation(
        &self,
        constitution: &Constitution,
        channel: Channel,
        latent: &[f64],
    ) -> (f64, f64) {
        let law = constitution.channel(channel);

        (
            law.bias(),
            law.latent_term(latent, self.sensitivity_change(channel)),
        )
    }

    /// Sensitisation or habituation after an occurrence.
    pub(crate) fn repeat(
        &mut self,
        constitution: &Constitution,
        signature: &str,
        law: &ReactionLaw,
        latent: &[f64],
    ) {
        let change = constitution.scales().gain_plasticity() * law.drift(latent);

        if change.is_finite() {
            let gain = self.gains.entry(signature.to_owned()).or_insert(0.0);

            *gain = (*gain + change).clamp(Self::GAIN_BOUNDS.0, Self::GAIN_BOUNDS.1);
        }
    }

    /// Slow changes of the coupling and of the resting point over lived
    /// time, from the state the individual was in.
    pub(crate) fn consolidate(
        &mut self,
        constitution: &Constitution,
        pre_activation: &[f64],
        hours: f64,
    ) {
        if !(hours.is_finite() && hours > 0.0) {
            return;
        }

        let scales = constitution.scales();
        let activity: Vec<f64> = pre_activation.iter().map(|value| value.tanh()).collect();
        let rate = scales.coupling_polarity() * scales.coupling_plasticity() * hours;
        let dimensions = activity.len();

        for row in 0..dimensions {
            for column in 0..dimensions {
                if row != column {
                    self.coupling
                        .add(row, column, rate * activity[row] * activity[column]);
                }
            }
        }

        let limit = Self::COUPLING_BOUND * constitution.coupling().frobenius().max(0.1);
        let norm = self.coupling.frobenius();

        if !self.coupling.is_finite() {
            self.coupling = SquareMatrix::zero(dimensions);
        } else if norm > limit {
            self.coupling.scale(limit / norm);
        }

        let share = 1.0 - (-scales.baseline_plasticity() * hours).exp();

        for (index, (change, born)) in self
            .baseline
            .iter_mut()
            .zip(constitution.baseline())
            .enumerate()
        {
            let effective = born + *change;
            let lived = pre_activation.get(index).copied().unwrap_or(effective);
            let target = (effective + share * (lived - effective))
                .clamp(-Self::BASELINE_BOUND, Self::BASELINE_BOUND);

            if target.is_finite() {
                *change = target - born;
            }
        }

        self.consolidated_hours += hours;
    }

    /// Habit: the sensitivity of the channel of an engaged mechanism moves
    /// towards the change of state that followed the engagement.
    pub(crate) fn engage(
        &mut self,
        constitution: &Constitution,
        channel: Channel,
        before: &[f64],
        after: &[f64],
    ) {
        let scales = constitution.scales();
        let rate = scales.habit_plasticity();
        let limit = Self::SENSITIVITY_BOUND * scales.sensitivity_scale();
        let change = self
            .sensitivities
            .entry(channel)
            .or_insert_with(|| vec![0.0; before.len()]);

        for ((value, before), after) in change.iter_mut().zip(before).zip(after) {
            let next = *value + rate * (after - before);

            if next.is_finite() {
                *value = next;
            }
        }

        let norm = change.iter().map(|value| value * value).sum::<f64>().sqrt();

        if norm > limit {
            change.iter_mut().for_each(|value| *value *= limit / norm);
        }

        self.engagements += 1;
    }
}
