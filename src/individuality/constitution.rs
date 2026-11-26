use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    individuality::{
        birth_parameters::BirthParameters, channel::Channel, channel_law::ChannelLaw,
        constitution_scales::ConstitutionScales, reaction_law::ReactionLaw,
    },
    mathematics::{
        random_source::RandomSource, square_matrix::SquareMatrix, stable_hash::StableHash,
    },
};

/// Laws an individual is born with. They are drawn once, at birth, from
/// the birth parameters, and never drawn again: what changes during the
/// existence is kept apart, in the dispositions (plasticity).
///
/// The latent dynamics they define (see `LatentDynamics`) is
///
/// `dh/dt = (b + W · tanh(h) − h) / τ + fluctuations`, `z = tanh(h)`,
///
/// with a relaxation time `τ` per dimension, a resting point `b`, an
/// internal coupling `W` (the interactions between dimensions) and
/// fluctuations of individual intensity. Occurrences add impulses to `h`
/// through reaction laws. Channels read `z`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Constitution {
    birth: BirthParameters,
    scales: ConstitutionScales,
    time_scales: Vec<f64>,
    baseline: Vec<f64>,
    coupling: SquareMatrix,
    noise: Vec<f64>,
    initial: Vec<f64>,
    channels: BTreeMap<Channel, ChannelLaw>,
    sensations: Vec<Vec<f64>>,
}

impl Constitution {
    /// Number of anonymous projections of the latent state that
    /// introspection can perceive.
    pub(crate) const SENSATIONS: usize = 6;
    /// Shortest relaxation time, so that integration stays meaningful.
    const MINIMUM_TIME_SCALE: f64 = 1.0;

    pub(crate) fn generate(birth: BirthParameters) -> Self {
        let scales = ConstitutionScales::draw(&mut RandomSource::new(StableHash::of(
            birth.seed(),
            "scales",
        )));

        Self::from_scales(birth, scales)
    }

    /// Laws drawn from given scales (birth, or a controlled variation of
    /// the scales of an existing birth).
    pub(crate) fn from_scales(birth: BirthParameters, scales: ConstitutionScales) -> Self {
        let dimensions = birth.dimensions();
        let mut random = RandomSource::new(StableHash::of(birth.seed(), "laws"));
        let unit = 1.0 / (dimensions as f64).sqrt();

        let time_scales = (0..dimensions)
            .map(|_| {
                random
                    .log_normal(scales.time_scale(), scales.time_spread())
                    .max(Self::MINIMUM_TIME_SCALE)
            })
            .collect();
        let baseline: Vec<f64> = (0..dimensions)
            .map(|_| scales.baseline_scale() * random.normal())
            .collect();
        let coupling = SquareMatrix::from_fn(dimensions, |_, _| {
            scales.coupling_gain() * unit * random.normal()
        });
        let noise: Vec<f64> = (0..dimensions)
            .map(|_| random.log_normal(scales.noise(), 0.3))
            .collect();
        // The state at birth: near the resting point, displaced by one
        // draw of the fluctuations.
        let initial = baseline
            .iter()
            .zip(&noise)
            .map(|(baseline, noise)| baseline + noise * random.normal())
            .collect();
        let channels = Channel::ALL
            .iter()
            .map(|channel| {
                let bias = channel.centre() + scales.bias_spread() * random.normal();
                let sensitivity = (0..dimensions)
                    .map(|_| scales.sensitivity_scale() * unit * random.normal())
                    .collect();

                (*channel, ChannelLaw::new(bias, sensitivity))
            })
            .collect();
        let sensations = (0..Self::SENSATIONS)
            .map(|_| (0..dimensions).map(|_| random.normal()).collect())
            .collect();

        Self {
            birth,
            scales,
            time_scales,
            baseline,
            coupling,
            noise,
            initial,
            channels,
            sensations,
        }
    }

    pub(crate) fn birth(&self) -> &BirthParameters {
        &self.birth
    }

    pub(crate) fn scales(&self) -> &ConstitutionScales {
        &self.scales
    }

    pub(crate) fn dimensions(&self) -> usize {
        self.birth.dimensions()
    }

    pub(crate) fn time_scales(&self) -> &[f64] {
        &self.time_scales
    }

    pub(crate) fn baseline(&self) -> &[f64] {
        &self.baseline
    }

    pub(crate) fn coupling(&self) -> &SquareMatrix {
        &self.coupling
    }

    pub(crate) fn noise(&self) -> &[f64] {
        &self.noise
    }

    pub(crate) fn initial(&self) -> &[f64] {
        &self.initial
    }

    pub(crate) fn channel(&self, channel: Channel) -> &ChannelLaw {
        &self.channels[&channel]
    }

    pub(crate) fn sensations(&self) -> &[Vec<f64>] {
        &self.sensations
    }

    /// Experimental replacement of a channel bias.
    pub(crate) fn set_bias(&mut self, channel: Channel, bias: f64) {
        let law = &self.channels[&channel];

        self.channels
            .insert(channel, ChannelLaw::new(bias, law.sensitivity().to_vec()));
    }

    /// Reaction law of an occurrence, drawn from the seed and the
    /// signature. Signatures of one family (the part before the first
    /// colon, e.g. `invoke` in `invoke:logical_inference`) share part of
    /// their direct effect, so that related occurrences have related
    /// effects, in proportion to the individual's family share.
    pub(crate) fn reaction_law(&self, signature: &str) -> ReactionLaw {
        let dimensions = self.dimensions();
        let seed = self.birth.seed();
        let unit = 1.0 / (dimensions as f64).sqrt();
        let family = signature.split(':').next().unwrap_or(signature);
        let mut shared = RandomSource::new(StableHash::of(seed, &format!("family:{family}")));
        let mut own = RandomSource::new(StableHash::of(seed, &format!("reaction:{signature}")));
        let share = self.scales.family_share();

        let common: Vec<f64> = (0..dimensions).map(|_| unit * shared.normal()).collect();
        let direct = common
            .iter()
            .map(|common| share * common + (1.0 - share) * unit * own.normal())
            .collect();
        let strength = self.scales.impulse_scale() * own.log_normal(1.0, 0.5);
        let gate_in = (0..dimensions)
            .map(|_| self.scales.gating() * unit * own.normal())
            .collect();
        let gate_out = (0..dimensions).map(|_| unit * own.normal()).collect();
        let drift = (0..dimensions).map(|_| unit * own.normal()).collect();

        ReactionLaw::new(strength, direct, gate_in, gate_out, drift)
    }
}

#[cfg(test)]
mod tests {
    use super::Constitution;
    use crate::individuality::{birth_parameters::BirthParameters, channel::Channel};

    #[test]
    fn a_birth_is_reproducible_and_individual() {
        let birth = |seed| BirthParameters::new(seed, 8).unwrap();
        let first = Constitution::generate(birth(1));

        assert_eq!(first, Constitution::generate(birth(1)));
        assert_ne!(first, Constitution::generate(birth(2)));
        assert_eq!(
            first.reaction_law("invoke:x"),
            first.reaction_law("invoke:x")
        );
        assert_ne!(
            first.reaction_law("invoke:x"),
            Constitution::generate(birth(2)).reaction_law("invoke:x")
        );
        assert!(
            Channel::ALL
                .iter()
                .all(|channel| first.channel(*channel).sensitivity().len() == 8)
        );
        assert!(BirthParameters::new(1, 1).is_err());
    }

    #[test]
    fn related_occurrences_have_related_effects() {
        let constitution = Constitution::generate(BirthParameters::new(5, 32).unwrap());
        let cosine = |left: &str, right: &str| {
            let left = constitution.reaction_law(left);
            let right = constitution.reaction_law(right);
            let dot: f64 = left
                .direct()
                .iter()
                .zip(right.direct())
                .map(|(a, b)| a * b)
                .sum();
            let norm =
                |values: &[f64]| values.iter().map(|value| value * value).sum::<f64>().sqrt();

            dot / (norm(left.direct()) * norm(right.direct()))
        };

        let mut related = 0.0;
        let mut unrelated = 0.0;

        for index in 0..20 {
            related += cosine(&format!("invoke:a{index}"), &format!("invoke:b{index}"));
            unrelated += cosine(&format!("invoke:a{index}"), &format!("answer:b{index}")).abs();
        }

        assert!(related > unrelated, "{related} {unrelated}");
    }
}
