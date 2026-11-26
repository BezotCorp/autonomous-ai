/// Deterministic pseudo-random generator (SplitMix64). A seed makes every
/// exploration decision reproducible, which is required for comparative
/// experiments.
#[derive(Clone, Debug)]
pub(crate) struct RandomSource {
    state: u64,
}

impl RandomSource {
    pub(crate) fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);

        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);

        value ^ (value >> 31)
    }

    /// Uniform sample in the open interval (0, 1).
    pub(crate) fn uniform(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64 + 0.5) / (1_u64 << 53) as f64
    }

    /// Standard normal sample (Box-Muller transform).
    pub(crate) fn normal(&mut self) -> f64 {
        let radius = (-2.0 * self.uniform().ln()).sqrt();
        let angle = 2.0 * std::f64::consts::PI * self.uniform();

        radius * angle.cos()
    }

    /// Gamma(shape, 1) sample (Marsaglia and Tsang method).
    pub(crate) fn gamma(&mut self, shape: f64) -> f64 {
        if shape < 1.0 {
            let boost = self.uniform().powf(1.0 / shape);

            return self.gamma(shape + 1.0) * boost;
        }

        let offset = shape - 1.0 / 3.0;
        let scale = 1.0 / (9.0 * offset).sqrt();

        loop {
            let normal = self.normal();
            let candidate = 1.0 + scale * normal;

            if candidate <= 0.0 {
                continue;
            }

            let cube = candidate * candidate * candidate;
            let uniform = self.uniform();

            if uniform.ln() < 0.5 * normal * normal + offset - offset * cube + offset * cube.ln() {
                return offset * cube;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RandomSource;

    #[test]
    fn is_reproducible_and_bounded() {
        let mut first = RandomSource::new(7);
        let mut second = RandomSource::new(7);

        for _ in 0..1000 {
            let value = first.uniform();

            assert_eq!(value, second.uniform());
            assert!(value > 0.0 && value < 1.0);
        }

        let mean = (0..20_000).map(|_| first.gamma(3.0)).sum::<f64>() / 20_000.0;

        assert!((mean - 3.0).abs() < 0.1);
    }
}
