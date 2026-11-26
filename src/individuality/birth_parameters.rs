use serde::{Deserialize, Serialize};

/// What a birth is reproduced from: the seed, the dimension of the latent
/// space and the version of the generator of constitutions. The same
/// parameters always produce the same constitution.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct BirthParameters {
    seed: u64,
    dimensions: usize,
    generator: String,
}

impl BirthParameters {
    /// Version of the generator; changing how constitutions are drawn
    /// requires a new version, so that old births stay reproducible.
    pub(crate) const GENERATOR: &'static str = "constitution-1";
    pub(crate) const DEFAULT_DIMENSIONS: usize = 12;
    pub(crate) const MAXIMUM_DIMENSIONS: usize = 64;

    pub(crate) fn new(seed: u64, dimensions: usize) -> Result<Self, String> {
        if !(2..=Self::MAXIMUM_DIMENSIONS).contains(&dimensions) {
            return Err(format!(
                "a latent space needs between 2 and {} dimensions, not {dimensions}",
                Self::MAXIMUM_DIMENSIONS
            ));
        }

        Ok(Self {
            seed,
            dimensions,
            generator: Self::GENERATOR.to_owned(),
        })
    }

    pub(crate) fn seed(&self) -> u64 {
        self.seed
    }

    pub(crate) fn dimensions(&self) -> usize {
        self.dimensions
    }
}
