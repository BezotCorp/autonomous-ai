/// Module able to act on the current state, with the features describing
/// the situation it would act on and what remains to gain from acting
/// (1 when something is unknown, the staleness of what it would verify).
#[derive(Debug)]
pub(crate) struct Candidate {
    module: usize,
    name: String,
    features: Vec<String>,
    stake: f64,
}

impl Candidate {
    pub(crate) fn new(module: usize, name: String, features: Vec<String>, stake: f64) -> Self {
        Self {
            module,
            name,
            features,
            stake,
        }
    }

    pub(crate) fn module(&self) -> usize {
        self.module
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn features(&self) -> &[String] {
        &self.features
    }

    pub(crate) fn stake(&self) -> f64 {
        self.stake
    }
}
