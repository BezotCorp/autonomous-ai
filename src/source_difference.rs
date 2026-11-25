use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SourceDifference {
    path: String,
    category: String,
    previous: Vec<String>,
    current: Vec<String>,
}

impl SourceDifference {
    pub(crate) fn path(&self) -> &str {
        &self.path
    }

    pub(crate) fn previous(&self) -> &[String] {
        &self.previous
    }

    pub(crate) fn current(&self) -> &[String] {
        &self.current
    }

    pub(crate) fn new(
        path: String,
        category: String,
        previous: Vec<String>,
        current: Vec<String>,
    ) -> Self {
        Self {
            path,
            category,
            previous,
            current,
        }
    }

    pub(crate) fn category(&self) -> &str {
        &self.category
    }
}
