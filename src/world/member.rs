use serde::{Deserialize, Serialize};

/// An existence present in an environment. Its handle is its operational
/// identity there: an opaque address, distinct from the historical identity
/// recorded inside the existence (copies of one existence share that one).
/// Nothing in the existence is changed to give it a handle.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Member {
    handle: String,
    individual: String,
    joined_at: f64,
    departed_at: Option<f64>,
}

impl Member {
    pub(crate) fn new(handle: String, individual: String, joined_at: f64) -> Self {
        Self {
            handle,
            individual,
            joined_at,
            departed_at: None,
        }
    }

    pub(crate) fn handle(&self) -> &str {
        &self.handle
    }

    /// Directory name of the existence, under `<data>/individuals/`.
    pub(crate) fn individual(&self) -> &str {
        &self.individual
    }

    pub(crate) fn joined_at(&self) -> f64 {
        self.joined_at
    }

    pub(crate) fn is_present(&self) -> bool {
        self.departed_at.is_none()
    }

    pub(crate) fn depart(&mut self, at: f64) {
        self.departed_at = Some(at);
    }
}
