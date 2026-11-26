/// How lived time passes during a session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClockMode {
    /// Lived time is wall-clock time: waiting really waits.
    Real,
    /// Lived time is computed: waiting jumps ahead, and reasoning costs a
    /// nominal duration per invocation. Experiments run faster than real
    /// time and do not depend on the speed of the machine.
    Virtual,
}

impl ClockMode {
    /// Nominal lived duration of one module invocation in virtual time.
    pub(crate) const VIRTUAL_INVOCATION_SECONDS: f64 = 2.0;
}
