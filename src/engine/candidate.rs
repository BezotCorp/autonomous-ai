/// Module able to act on the current state, with the features describing
/// the situation it would act on.
#[derive(Debug)]
pub(crate) struct Candidate {
    module: usize,
    name: String,
    features: Vec<String>,
}

impl Candidate {
    pub(crate) fn new(module: usize, name: String, features: Vec<String>) -> Self {
        Self {
            module,
            name,
            features,
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
}
