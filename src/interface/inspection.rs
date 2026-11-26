use std::error::Error;

use serde::Serialize;

use crate::{
    individuality::{channel::Channel, channel_law::ChannelLaw},
    influences::influence_view::InfluenceView,
    living::{existence::Existence, life_event::LifeEvent, life_summary::LifeSummary},
    memory::{episode_summary::EpisodeSummary, learning_statistics::LearningStatistics},
};

/// Renders a section of a saved existence. Inspection reads a saved state
/// and never touches the individual.
pub(crate) struct Inspection;

impl Inspection {
    pub(crate) fn render(
        existence: &Existence,
        section: &str,
        last: Option<usize>,
        kind: Option<&str>,
        csv: bool,
        period: (f64, f64),
    ) -> Result<String, Box<dyn Error>> {
        let memory = existence.memory();
        let (from, to) = period;

        Ok(match section {
            "summary" if csv => format!(
                "{}\n{}",
                LifeSummary::csv_header(),
                LifeSummary::of(existence).csv_row()
            ),
            "summary" => LifeSummary::of(existence).to_string(),
            "report" => {
                crate::living::period_report::PeriodReport::of(existence, from, to).to_string()
            }
            "identity" => Self::ron(existence.identity())?,
            "constitution" => Self::ron(existence.constitution())?,
            "dispositions" => Self::ron(existence.dispositions())?,
            "latent" => Self::ron(&(
                existence.latent().pre(),
                existence.latent().activation(),
                existence.latent().sensations(existence.constitution()),
            ))?,
            "channels" => {
                let latent = existence.latent().activation();
                let mut lines =
                    vec!["channel  bias  latent_term  acquired_sensitivity_norm  value".to_owned()];

                for channel in Channel::ALL {
                    let (bias, term) = existence.dispositions().activation(
                        existence.constitution(),
                        channel,
                        &latent,
                    );
                    let acquired = existence
                        .dispositions()
                        .sensitivity_change(channel)
                        .map_or(0.0, |change| {
                            change.iter().map(|value| value * value).sum::<f64>().sqrt()
                        });

                    lines.push(format!(
                        "{channel} {bias:.4} {term:.4} {acquired:.4} {:.6}",
                        ChannelLaw::shape(channel.form(), bias + term)
                    ));
                }

                lines.join("\n")
            }
            "trajectory" => {
                let dimensions = existence.constitution().dimensions();
                let mut lines = vec![format!(
                    "lived,{}",
                    (1..=dimensions)
                        .map(|index| format!("z{index}"))
                        .collect::<Vec<_>>()
                        .join(",")
                )];

                for entry in existence.timeline().entries() {
                    if let LifeEvent::Latent { state } = entry.event() {
                        lines.push(format!(
                            "{:.1},{}",
                            entry.lived(),
                            state
                                .iter()
                                .map(|value| format!("{value:.5}"))
                                .collect::<Vec<_>>()
                                .join(",")
                        ));
                    }
                }

                lines.join("\n")
            }
            "timeline" => {
                let entries: Vec<_> = existence
                    .timeline()
                    .entries()
                    .iter()
                    .filter(|entry| kind.is_none_or(|kind| entry.event().kind() == kind))
                    .filter(|entry| entry.lived() >= from && entry.lived() <= to)
                    .collect();
                let skip = entries.len().saturating_sub(last.unwrap_or(200));

                Self::ron(&(
                    existence.timeline().totals(),
                    existence.timeline().dropped(),
                    &entries[skip..],
                ))?
            }
            "activities" => Self::ron(existence.activities())?,
            "concerns" => Self::ron(existence.concerns())?,
            "stimuli" => Self::ron(existence.stimuli())?,
            "reactions" => {
                let mut reactions: Vec<_> = existence
                    .catalog()
                    .reactions()
                    .iter()
                    .map(|(signature, record)| {
                        (
                            signature.clone(),
                            record.count(),
                            record.mean_norm(),
                            existence.dispositions().gain(signature),
                            record.mean_impulse(),
                        )
                    })
                    .collect();

                reactions.sort_by_key(|reaction| std::cmp::Reverse(reaction.1));
                Self::ron(&reactions)?
            }
            "sequences" => Self::ron(existence.catalog().sequences())?,
            "episodes" => {
                let episodes = memory.experiences().episodes();
                let skip = episodes
                    .len()
                    .saturating_sub(last.unwrap_or(episodes.len()));
                let mut lines = vec![EpisodeSummary::HEADER.to_owned()];

                for (index, episode) in episodes.iter().enumerate().skip(skip) {
                    lines.push(EpisodeSummary::new(index + 1, episode).to_string());
                }

                lines.join("\n")
            }
            "statistics" => Self::ron(&LearningStatistics::compute(memory))?,
            "knowledge" => Self::ron(&memory.knowledge().entries().collect::<Vec<_>>())?,
            "hypotheses" => Self::ron(&memory.knowledge().hypotheses().collect::<Vec<_>>())?,
            "scopes" => Self::ron(&memory.knowledge().scopes().collect::<Vec<_>>())?,
            "routing" => Self::ron(memory.routing())?,
            "resources" => Self::ron(memory.resources())?,
            "experiences" => Self::ron(memory.experiences())?,
            "dreams" => Self::ron(memory.dreams())?,
            "corrections" => Self::ron(memory.corrections())?,
            "influences" => {
                let evidence = memory.influences().evidence();
                let views: Vec<InfluenceView> = memory
                    .influences()
                    .influences()
                    .iter()
                    .map(|influence| {
                        InfluenceView::new(
                            influence,
                            evidence.get(&influence.id()).copied().unwrap_or_default(),
                        )
                    })
                    .collect();

                Self::ron(&views)?
            }
            other => return Err(format!("Unknown section {other}").into()),
        })
    }

    fn ron<T: Serialize + ?Sized>(value: &T) -> Result<String, Box<dyn Error>> {
        Ok(ron::ser::to_string_pretty(
            value,
            ron::ser::PrettyConfig::default(),
        )?)
    }
}
