/// Prior belief about how often an observed scope changes, as if `changes`
/// changes had been seen over `seconds` seconds. It only matters until the
/// scope has been verified a few times.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ChangePrior {
    changes: f64,
    seconds: f64,
}

impl ChangePrior {
    pub(crate) fn new(changes: f64, seconds: f64) -> Self {
        Self { changes, seconds }
    }

    pub(crate) fn changes(&self) -> f64 {
        self.changes
    }

    pub(crate) fn seconds(&self) -> f64 {
        self.seconds
    }
}
