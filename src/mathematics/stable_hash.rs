/// 64-bit FNV-1a hash. Unlike the standard hasher, its value never changes
/// between versions of the compiler or of the program, so that laws derived
/// from a hash (for instance the reaction an individual has to an action it
/// never met before) are reproducible across executions.
pub(crate) struct StableHash;

impl StableHash {
    const OFFSET: u64 = 0xCBF2_9CE4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01B3;

    pub(crate) fn of(seed: u64, text: &str) -> u64 {
        let mut hash = Self::OFFSET;

        for byte in seed.to_le_bytes().iter().chain(text.as_bytes()) {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(Self::PRIME);
        }

        hash
    }
}

#[cfg(test)]
mod tests {
    use super::StableHash;

    #[test]
    fn is_stable_and_discriminating() {
        assert_eq!(StableHash::of(0, ""), 0xA8C7_F832_281A_39C5);
        assert_eq!(StableHash::of(1, "a"), StableHash::of(1, "a"));
        assert_ne!(StableHash::of(1, "a"), StableHash::of(2, "a"));
        assert_ne!(StableHash::of(1, "a"), StableHash::of(1, "b"));
    }
}
