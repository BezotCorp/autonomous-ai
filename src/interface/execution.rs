use std::{
    error::Error,
    io::{self, Read, Write},
    path::Path,
    process::ExitCode,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

use crate::{
    capabilities::module_catalog::ModuleCatalog,
    engine::{
        autonomy::Autonomy, intelligence::Intelligence, learning_parameters::LearningParameters,
        memory_correction::MemoryCorrection,
    },
    events::event::Event,
    influences::influence_view::InfluenceView,
    interface::{arguments::Arguments, output::Output},
    memory::{
        episode_configuration::EpisodeConfiguration, episode_summary::EpisodeSummary,
        learning_statistics::LearningStatistics, memory_store::MemoryStore,
    },
    resources::initial_resources::InitialResources,
};

pub(crate) fn execute() -> ExitCode {
    match execute_inner() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn execute_inner() -> Result<(), Box<dyn Error>> {
    match Arguments::parse()? {
        Arguments::Run {
            memory,
            resources,
            seed,
            acquired_knowledge,
            learned_routing,
            offline,
            maximum_invocations,
            cost_weight,
            scarcity_weight,
            minimum_utility,
            budgets,
            acquisition_cache,
            influences,
            output,
        } => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            let event: Event = ron::de::from_str(&input)?;

            let resources = InitialResources::load(&resources)?;
            let mut parameters = LearningParameters::default();

            if let Some(maximum) = maximum_invocations {
                parameters = parameters.with_maximum_invocations(maximum);
            }

            if let Some(weight) = cost_weight {
                parameters = parameters.with_cost_weight_per_second(weight);
            }

            if let Some(weight) = scarcity_weight {
                parameters = parameters.with_scarcity_weight(weight);
            }

            if let Some(minimum) = minimum_utility {
                parameters = parameters.with_minimum_utility(minimum);
            }

            let seed = match seed {
                Some(seed) => seed,
                None => u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos())
                    .unwrap_or(u64::MAX),
            };

            let store = MemoryStore::open(&memory)?;
            let mut persistent = store.load()?;
            let intelligence =
                Intelligence::new(ModuleCatalog::standard(offline), resources, parameters);

            let report = intelligence.reason(
                &event,
                &mut persistent,
                {
                    let mut configuration =
                        EpisodeConfiguration::new(seed, acquired_knowledge, learned_routing)
                            .with_resource_budgets(budgets);

                    if !influences {
                        configuration = configuration.without_influences();
                    }

                    if acquisition_cache {
                        configuration
                    } else {
                        configuration.without_acquisition_cache()
                    }
                },
                output == Output::Detailed,
            );

            store.save(&persistent)?;

            if output == Output::Summary {
                let mut stdout = io::stdout().lock();

                writeln!(
                    stdout,
                    "{}",
                    EpisodeSummary::new(
                        persistent.experiences().episodes().len(),
                        report.episode()
                    )
                )?;
            } else {
                write_output(&report)?;
            }
        }

        Arguments::Idle {
            memory,
            resources,
            seed,
            offline,
            maximum_activities,
            reflection,
            dreams,
            cost_weight,
            minimum_worth,
        } => {
            let resources = InitialResources::load(&resources)?;
            let mut parameters = LearningParameters::default();

            if let Some(weight) = cost_weight {
                parameters = parameters.with_cost_weight_per_second(weight);
            }

            let seed = match seed {
                Some(seed) => seed,
                None => u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos())
                    .unwrap_or(u64::MAX),
            };

            let store = MemoryStore::open(&memory)?;
            let mut persistent = store.load()?;
            let intelligence = Intelligence::new(
                ModuleCatalog::standard(offline),
                resources,
                parameters.clone(),
            );
            let records = Autonomy::new(
                &intelligence,
                &parameters,
                reflection,
                dreams,
                minimum_worth,
            )
            .run(&mut persistent, maximum_activities, seed);

            store.save(&persistent)?;

            let mut stdout = io::stdout().lock();

            if records.is_empty() {
                writeln!(stdout, "nothing seemed worth doing")?;
            }

            for record in records {
                writeln!(
                    stdout,
                    "{}  {}  {:.2}s  {}  {}",
                    record.activity().label(),
                    record.focus(),
                    record.seconds(),
                    if record.worthwhile() {
                        "worthwhile"
                    } else {
                        "fruitless"
                    },
                    record.summary()
                )?;
            }
        }

        Arguments::Revert {
            memory,
            correction,
            reason,
        } => {
            let store = open_existing(&memory)?;
            let mut persistent = store.load()?;
            let id = correction
                .parse()
                .map_err(|_| format!("Invalid correction identifier {correction}"))?;

            MemoryCorrection::revert(&mut persistent, id, reason)?;
            store.save(&persistent)?;
            writeln!(io::stdout().lock(), "correction {id} reverted")?;
        }

        Arguments::Inspect { memory, section } => {
            let persistent = open_existing(&memory)?.load()?;

            match section.as_str() {
                "summary" => {
                    let mut stdout = io::stdout().lock();

                    writeln!(stdout, "{}", EpisodeSummary::HEADER)?;

                    for (index, episode) in persistent.experiences().episodes().iter().enumerate() {
                        writeln!(stdout, "{}", EpisodeSummary::new(index + 1, episode))?;
                    }
                }
                "scopes" => write_output(&persistent.knowledge().scopes().collect::<Vec<_>>())?,
                "resources" => write_output(persistent.resources())?,
                "dreams" => write_output(persistent.dreams())?,
                "corrections" => write_output(persistent.corrections())?,
                "activities" => write_output(persistent.activities())?,
                "influences" => {
                    let evidence = persistent.influences().evidence();
                    let views: Vec<InfluenceView> = persistent
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

                    write_output(&views)?
                }
                "knowledge" => write_output(&persistent.knowledge().entries().collect::<Vec<_>>())?,
                "hypotheses" => {
                    write_output(&persistent.knowledge().hypotheses().collect::<Vec<_>>())?
                }
                "routing" => write_output(persistent.routing())?,
                "experiences" => write_output(persistent.experiences())?,
                "statistics" => write_output(&LearningStatistics::compute(&persistent))?,
                other => {
                    return Err(format!("Unknown section {other}\n{}", Arguments::USAGE).into());
                }
            }
        }

        Arguments::Relearn { memory } => {
            let store = open_existing(&memory)?;
            let mut persistent = store.load()?;

            persistent.relearn_routing();
            store.save(&persistent)?;
            write_output(persistent.routing())?;
        }

        Arguments::Modules => {
            let registry = ModuleCatalog::standard(false);
            let descriptors: Vec<_> = registry
                .modules()
                .iter()
                .map(|module| module.descriptor())
                .collect();

            write_output(&descriptors)?;
        }
    }

    Ok(())
}

fn open_existing(path: &Path) -> Result<MemoryStore, Box<dyn Error>> {
    if !path.is_file() {
        return Err(format!("Memory file does not exist: {}", path.display()).into());
    }

    MemoryStore::open(path)
}

fn write_output<T: Serialize + ?Sized>(value: &T) -> Result<(), Box<dyn Error>> {
    let output = ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::default())?;
    let mut stdout = io::stdout().lock();

    writeln!(stdout, "{output}")?;

    Ok(())
}
