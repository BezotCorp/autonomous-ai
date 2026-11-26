use serde::{Deserialize, Serialize};

/// One action of a plan: mobilising a module for the information it is
/// expected to supply, once the steps it depends on have been carried out.
/// Every need a module is expected to supply is grouped into one step, so
/// that it can acquire everything in a single invocation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct PlanStep {
    module: String,
    supplies: Vec<String>,
    depends_on: Vec<usize>,
}

impl PlanStep {
    pub(crate) fn new(module: String, supplies: Vec<String>, depends_on: Vec<usize>) -> Self {
        Self {
            module,
            supplies,
            depends_on,
        }
    }

    pub(crate) fn module(&self) -> &str {
        &self.module
    }

    pub(crate) fn depends_on(&self) -> &[usize] {
        &self.depends_on
    }
}
