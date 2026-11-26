use std::fmt;

use crate::{
    memory::{answer_status::AnswerStatus, episode_record::EpisodeRecord},
    planning::plan_status::PlanStatus,
};

/// One-line account of an episode, to follow long experiments without
/// reading complete reports.
pub(crate) struct EpisodeSummary<'a> {
    index: usize,
    episode: &'a EpisodeRecord,
}

impl<'a> EpisodeSummary<'a> {
    pub(crate) const HEADER: &'static str = "#  started  event  answers(proven/refuted/undetermined)  currency  invocations  seconds  requests(units)  avoided  redundant  refused  plans(adopted/revised/achieved)  memory(recalled/in-answers/confirmed/superseded)  termination  configuration";

    pub(crate) fn new(index: usize, episode: &'a EpisodeRecord) -> Self {
        Self { index, episode }
    }
}

impl fmt::Display for EpisodeSummary<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let episode = self.episode;
        let count = |status: AnswerStatus| {
            episode
                .answers()
                .iter()
                .filter(|answer| answer.status() == status)
                .count()
        };
        let historical: Vec<_> = episode
            .answers()
            .iter()
            .filter(|answer| answer.status() != AnswerStatus::Undetermined && !answer.is_current())
            .collect();
        let currency = if historical.is_empty() {
            "current".to_owned()
        } else {
            format!(
                "historical({} max-staleness={:.3})",
                historical.len(),
                historical
                    .iter()
                    .map(|answer| answer.staleness())
                    .fold(0.0, f64::max)
            )
        };
        let usage = episode.usage();
        let requests: Vec<String> = usage
            .requests()
            .iter()
            .map(|(resource, requests)| {
                format!(
                    "{resource}={requests}({})",
                    usage.consumption().get(resource).copied().unwrap_or(0.0)
                )
            })
            .collect();
        let plans = episode.plans();
        let revised = plans
            .iter()
            .filter(|record| record.status() == PlanStatus::Revised)
            .count();
        let achieved = plans.iter().filter(|record| record.achieved()).count();
        let exploitation = episode.exploitation();

        write!(
            formatter,
            "{}  {}  {}  {}/{}/{}  {}  {}  {:.2}  {}  {}  {}  {}  {}/{}/{}  {}/{}/{}/{}  {:?}  {}",
            self.index,
            episode.started_at().format("%Y-%m-%dT%H:%M:%S"),
            episode.event_id(),
            count(AnswerStatus::Proven),
            count(AnswerStatus::Refuted),
            count(AnswerStatus::Undetermined) + count(AnswerStatus::Contradictory),
            currency,
            episode.invocations().len(),
            episode.duration_micros() as f64 / 1e6,
            if requests.is_empty() {
                "none".to_owned()
            } else {
                requests.join(",")
            },
            usage.avoided(),
            usage.redundant(),
            usage.refused(),
            plans.len(),
            revised,
            achieved,
            exploitation.recall_count(),
            exploitation.recalled_in_answers(),
            exploitation.confirmed_count(),
            exploitation.superseded_count(),
            episode.termination(),
            episode.configuration().label(),
        )
    }
}
