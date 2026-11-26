use serde::{Deserialize, Serialize};

/// Something that happened to or in the individual: an action it took, a
/// reaction it met, a stimulus, an activity beginning or ending, a
/// concern forming. Occurrences are identified by an open signature
/// (`family:detail`), never by a closed list: an occurrence met for the
/// first time receives its own reaction law. Its magnitude scales the
/// effect (1 for a plain occurrence).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Occurrence {
    signature: String,
    magnitude: f64,
}

impl Occurrence {
    /// Bound on the magnitude of one occurrence.
    const MAXIMUM_MAGNITUDE: f64 = 10.0;

    pub(crate) fn new(signature: impl Into<String>, magnitude: f64) -> Self {
        Self {
            signature: signature.into(),
            magnitude: if magnitude.is_finite() {
                magnitude.clamp(0.0, Self::MAXIMUM_MAGNITUDE)
            } else {
                0.0
            },
        }
    }

    pub(crate) fn plain(signature: impl Into<String>) -> Self {
        Self::new(signature, 1.0)
    }

    pub(crate) fn signature(&self) -> &str {
        &self.signature
    }

    pub(crate) fn magnitude(&self) -> f64 {
        self.magnitude
    }

    pub(crate) fn family(&self) -> &str {
        self.signature.split(':').next().unwrap_or(&self.signature)
    }
}
