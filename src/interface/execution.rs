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
    engine::{intelligence::Intelligence, learning_parameters::LearningParameters},
    events::event::Event,
    interface::arguments::Arguments,
    memory::{
        episode_configuration::EpisodeConfiguration, learning_statistics::LearningStatistics,
        memory_store::MemoryStore,
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
            detailed,
        } => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            let event: Event = ron::de::from_str(&input)?;

            let resources = InitialResources::load(&resources)?;
            let mut parameters = LearningParameters::default();

            if let Some(maximum) = maximum_invocations {
                parameters = parameters.with_maximum_invocations(maximum);
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
                EpisodeConfiguration::new(seed, acquired_knowledge, learned_routing),
                detailed,
            );

            store.save(&persistent)?;
            write_output(&report)?;
        }

        Arguments::Inspect { memory, section } => {
            let persistent = open_existing(&memory)?.load()?;

            match section.as_str() {
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
