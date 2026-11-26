/// Stable fingerprint of data (64-bit FNV-1a), used as the revision of a
/// source that does not provide one, so that unchanged data is recognised
/// from one acquisition to the next.
pub(crate) struct Fingerprint;

impl Fingerprint {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    pub(crate) fn of<'a>(parts: impl IntoIterator<Item = &'a str>) -> String {
        let mut hash = Self::OFFSET;

        for part in parts {
            for byte in part.bytes().chain([0]) {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(Self::PRIME);
            }
        }

        format!("{hash:016x}")
    }
}

#[cfg(test)]
mod tests {
    use super::Fingerprint;

    #[test]
    fn distinguishes_part_boundaries() {
        assert_eq!(Fingerprint::of(["a", "b"]), Fingerprint::of(["a", "b"]));
        assert_ne!(Fingerprint::of(["ab"]), Fingerprint::of(["a", "b"]));
    }
}
