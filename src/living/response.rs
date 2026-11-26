use serde::Serialize;

/// What the individual sends back after acting on a stimulus.
#[derive(Debug, Serialize)]
pub(crate) struct Response {
    event: String,
    lived: f64,
    answers: Vec<(String, String, Vec<String>)>,
}

impl Response {
    pub(crate) fn new(
        event: String,
        lived: f64,
        answers: Vec<(String, String, Vec<String>)>,
    ) -> Self {
        Self {
            event,
            lived,
            answers,
        }
    }
}
