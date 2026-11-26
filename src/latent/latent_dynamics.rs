use crate::{
    individuality::{constitution::Constitution, dispositions::Dispositions},
    latent::latent_state::LatentState,
    mathematics::random_source::RandomSource,
};

/// Integration of the latent dynamics over a lapse of time:
///
/// `dh/dt = (b + W · tanh(h) − h) / τ` plus Ornstein-Uhlenbeck fluctuations,
///
/// where `b` and `W` are the effective resting point and coupling
/// (constitution plus dispositions) and `τ` the relaxation times.
///
/// The exponential integrator `h ← T + (h − T) · e^(−dt/τ)`, with the
/// target `T = b + W · tanh(h)` frozen over a step, is stable for any step
/// (each step moves `h` towards a bounded target), which allows long lapses
/// (a technical stop of days) to be integrated in a bounded number of
/// steps. Fluctuations are added with their exact variance over a step,
/// `σ² (1 − e^(−2dt/τ))`.
pub(crate) struct LatentDynamics;

impl LatentDynamics {
    /// Bound on the steps of one integration; longer lapses take longer steps.
    const MAXIMUM_STEPS: usize = 2_000;

    /// Advances the state by `seconds`. Without a random source (a
    /// technical stop), only the deterministic part acts.
    pub(crate) fn advance(
        state: &mut LatentState,
        constitution: &Constitution,
        dispositions: &Dispositions,
        seconds: f64,
        mut random: Option<&mut RandomSource>,
    ) {
        if !(seconds.is_finite() && seconds > 0.0) {
            return;
        }

        let baseline = dispositions.baseline(constitution);
        let coupling = dispositions.coupling(constitution);
        let time_scales = constitution.time_scales();
        let noise = constitution.noise();
        let shortest = time_scales.iter().copied().fold(f64::INFINITY, f64::min);
        let step = (shortest / 4.0).max(seconds / Self::MAXIMUM_STEPS as f64);
        let mut remaining = seconds;

        while remaining > 0.0 {
            let lapse = step.min(remaining);
            let drive = coupling.apply(&state.activation());
            let next: Vec<f64> = state
                .pre()
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    let target = baseline[index] + drive[index];
                    let decay = (-lapse / time_scales[index]).exp();
                    let fluctuation = random.as_deref_mut().map_or(0.0, |random| {
                        noise[index] * (1.0 - decay * decay).max(0.0).sqrt() * random.normal()
                    });

                    target + (value - target) * decay + fluctuation
                })
                .collect();

            state.set_pre(next, &baseline);
            remaining -= lapse;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LatentDynamics;
    use crate::{
        individuality::{
            birth_parameters::BirthParameters, constitution::Constitution,
            constitution_scales::ConstitutionScales, dispositions::Dispositions,
        },
        latent::latent_state::LatentState,
        mathematics::random_source::RandomSource,
    };

    fn extreme(seed: u64, scale: &str, factor: f64) -> Constitution {
        let birth = BirthParameters::new(seed, 16).unwrap();
        let scales = Constitution::generate(birth.clone()).scales().clone();

        Constitution::from_scales(
            birth,
            ConstitutionScales::scaled(scales, scale, factor).unwrap(),
        )
    }

    #[test]
    fn stays_finite_and_bounded_for_extreme_constitutions() {
        let mut random = RandomSource::new(9);

        for (scale, factor) in [
            ("coupling_gain", 1_000.0),
            ("coupling_gain", 0.0),
            ("noise", 1e6),
            ("time_scale", 1e-9),
            ("time_scale", 1e12),
            ("baseline_scale", 1e9),
        ] {
            for seed in 0..5 {
                let constitution = extreme(seed, scale, factor);
                let dispositions = Dispositions::new(&constitution);
                let mut state = LatentState::new(constitution.initial());

                for lapse in [0.1, 1.0, 3_600.0, 1e9, f64::INFINITY, f64::NAN, -5.0] {
                    LatentDynamics::advance(
                        &mut state,
                        &constitution,
                        &dispositions,
                        lapse,
                        Some(&mut random),
                    );

                    assert!(
                        state
                            .pre()
                            .iter()
                            .all(|value| value.is_finite() && value.abs() <= LatentState::BOUND),
                        "{scale} x{factor}: {state:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn relaxes_without_fluctuations_during_a_technical_stop() {
        let constitution = extreme(3, "coupling_gain", 0.0);
        let dispositions = Dispositions::new(&constitution);
        let mut state = LatentState::new(&[LatentState::BOUND; 16]);

        LatentDynamics::advance(&mut state, &constitution, &dispositions, 1e9, None);

        for (value, baseline) in state.pre().iter().zip(constitution.baseline()) {
            assert!((value - baseline.clamp(-6.0, 6.0)).abs() < 1e-6);
        }
    }
}
