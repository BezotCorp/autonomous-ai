/// What `run` prints once the episode is over.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Output {
    /// One line: answers, currency, effort, requests, plans, memory use.
    Summary,
    /// The episode record and the evolution of knowledge.
    Report,
    /// The report and the whole reasoning graph.
    Detailed,
}
