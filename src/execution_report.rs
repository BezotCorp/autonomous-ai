use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct ExecutionReport {
    event_id: String,
    duplicate: bool,
    observations_added: usize,
    failures_added: usize,
    total_events: usize,
    total_observations: usize,
}

impl ExecutionReport {
    pub(crate) fn new(
        event_id: String,
        duplicate: bool,
        observations_added: usize,
        failures_added: usize,
        total_events: usize,
        total_observations: usize,
    ) -> Self {
        Self {
            event_id,
            duplicate,
            observations_added,
            failures_added,
            total_events,
            total_observations,
        }
    }
}
