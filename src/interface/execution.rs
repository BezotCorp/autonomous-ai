use std::{
    collections::BTreeSet,
    error::Error,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    capabilities::module_catalog::ModuleCatalog,
    engine::{
        intelligence::Intelligence, learning_parameters::LearningParameters,
        memory_correction::MemoryCorrection, neutral_disposition::NeutralDisposition,
    },
    events::event::Event,
    individuality::{birth_parameters::BirthParameters, channel::Channel},
    interface::{command_line::CommandLine, inspection::Inspection, output::Output},
    living::{
        clock_mode::ClockMode, existence::Existence, existence_store::ExistenceStore, life::Life,
        life_event::LifeEvent, life_settings::LifeSettings, life_summary::LifeSummary,
        mailbox::Mailbox, scheduled_stimulus::ScheduledStimulus,
    },
    memory::{
        activity::Activity, episode_configuration::EpisodeConfiguration,
        episode_summary::EpisodeSummary,
    },
    resources::initial_resources::InitialResources,
};

pub(crate) fn execute() -> ExitCode {
    match CommandLine::parse(std::env::args().skip(1))
        .map_err(Into::into)
        .and_then(|line| execute_command(&line))
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn data(line: &CommandLine) -> PathBuf {
    PathBuf::from(line.value("--data").unwrap_or("data"))
}

fn individual(line: &CommandLine) -> Result<PathBuf, Box<dyn Error>> {
    let name = line.positional(0, "the name of the individual")?;

    if name.is_empty() || name.contains(['/', '\\']) || name.starts_with('.') {
        return Err(format!("Invalid individual name {name}").into());
    }

    Ok(data(line).join("individuals").join(name))
}

fn resources(line: &CommandLine) -> Result<InitialResources, Box<dyn Error>> {
    let directory = line
        .value("--resources")
        .map(PathBuf::from)
        .unwrap_or_else(|| data(line).join("resources"));

    InitialResources::load(&directory)
}

fn clock_seed() -> Result<u64, Box<dyn Error>> {
    Ok(u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()).unwrap_or(u64::MAX))
}

fn existing(directory: &Path) -> Result<(ExistenceStore, Existence), Box<dyn Error>> {
    let store = ExistenceStore::open(directory)?;

    if !store.exists() {
        return Err(format!(
            "No individual in {}: give birth to it first",
            directory.display()
        )
        .into());
    }

    let existence = store.load()?;

    Ok((store, existence))
}

fn read_stdin() -> Result<String, Box<dyn Error>> {
    let mut input = String::new();

    io::stdin().read_to_string(&mut input)?;
    Ok(input)
}

fn print(text: impl std::fmt::Display) -> Result<(), Box<dyn Error>> {
    writeln!(io::stdout().lock(), "{text}")?;
    Ok(())
}

fn execute_command(line: &CommandLine) -> Result<(), Box<dyn Error>> {
    match line.command() {
        "birth" => birth(line),
        "live" => live(line),
        "stimulate" => {
            line.accept(&[], &[])?;

            let directory = individual(line)?;

            if !ExistenceStore::path(&directory).is_file() {
                return Err(format!("No individual in {}", directory.display()).into());
            }

            let raw = read_stdin()?;
            let event: Event = ron::de::from_str(&raw)?;
            let path = Mailbox::new(&directory).deliver(&event, &raw)?;

            print(format!("delivered to {}", path.display()))
        }
        "stop" => {
            line.accept(&[], &[])?;
            Mailbox::new(&individual(line)?).request_stop()?;
            print("stop requested")
        }
        "episode" => episode(line),
        "duplicate" => {
            line.accept(&["--reseed"], &[])?;

            let source = individual(line)?;
            let name = line.positional(1, "the name of the duplicate")?;
            let original = ExistenceStore::read(&ExistenceStore::path(&source))?;
            let target = data(line).join("individuals").join(name);
            let store = ExistenceStore::open(&target)?;

            if store.exists() {
                return Err(format!("{} already exists", target.display()).into());
            }

            store.save(&original.duplicate(name.to_owned(), line.number("--reseed")?))?;
            print(format!(
                "{} duplicated into {}",
                original.identity().name(),
                target.display()
            ))
        }
        "intervene" => intervene(line),
        "inspect" => {
            line.accept(&["--section", "--last", "--kind"], &["--csv"])?;

            let existence = ExistenceStore::read(&ExistenceStore::path(&individual(line)?))?;

            print(Inspection::render(
                &existence,
                line.value("--section").unwrap_or("summary"),
                line.number("--last")?,
                line.value("--kind"),
                line.flag("--csv"),
            )?)
        }
        "revert" => {
            line.accept(&["--correction", "--reason"], &[])?;

            let (store, mut existence) = existing(&individual(line)?)?;
            let correction = line
                .value("--correction")
                .ok_or(format!("--correction is required\n{}", CommandLine::USAGE))?;
            let id = correction
                .parse()
                .map_err(|_| format!("Invalid correction identifier {correction}"))?;

            MemoryCorrection::revert(
                existence.memory_mut(),
                id,
                line.value("--reason")
                    .unwrap_or("reverted on request")
                    .to_owned(),
            )?;
            existence.record(LifeEvent::Intervention {
                description: format!("correction {id} reverted by the experimenter"),
            });
            store.save(&existence)?;
            print(format!("correction {id} reverted"))
        }
        "relearn" => {
            line.accept(&[], &[])?;

            let (store, mut existence) = existing(&individual(line)?)?;

            existence.memory_mut().relearn_routing();
            existence.record(LifeEvent::Intervention {
                description: "routing relearned from the experiences".to_owned(),
            });
            store.save(&existence)?;
            print("routing relearned")
        }
        "individuals" => {
            line.accept(&[], &[])?;

            let directory = data(line).join("individuals");
            let mut names: Vec<PathBuf> = match fs::read_dir(&directory) {
                Ok(entries) => entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .collect(),
                Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
                Err(error) => return Err(error.into()),
            };

            names.sort();

            for path in names {
                let file = ExistenceStore::path(&path);

                if file.is_file() {
                    match ExistenceStore::read(&file) {
                        Ok(existence) => print(format!(
                            "{}  seed {}  lived {:.0} s  sessions {}  {}",
                            existence.identity().name(),
                            existence.identity().birth().seed(),
                            existence.clock().lived(),
                            existence.clock().sessions(),
                            existence.vigilance().label()
                        ))?,
                        Err(error) => print(format!("{}: {error}", path.display()))?,
                    }
                }
            }

            Ok(())
        }
        "modules" => {
            line.accept(&[], &[])?;

            let registry = ModuleCatalog::standard(false);
            let descriptors: Vec<_> = registry
                .modules()
                .iter()
                .map(|module| module.descriptor())
                .collect();

            print(ron::ser::to_string_pretty(
                &descriptors,
                ron::ser::PrettyConfig::default(),
            )?)
        }
        _ => Err(CommandLine::USAGE.into()),
    }
}

fn birth(line: &CommandLine) -> Result<(), Box<dyn Error>> {
    line.accept(&["--seed", "--dimensions"], &[])?;

    let directory = individual(line)?;
    let name = line.positional(0, "the name of the individual")?;
    let seed = match line.number("--seed")? {
        Some(seed) => seed,
        None => clock_seed()?,
    };
    let dimensions = line
        .number("--dimensions")?
        .unwrap_or(BirthParameters::DEFAULT_DIMENSIONS);
    let store = ExistenceStore::open(&directory)?;

    if store.exists() {
        return Err(format!(
            "{} already exists: an individual is born only once",
            directory.display()
        )
        .into());
    }

    let existence = Existence::born(name.to_owned(), BirthParameters::new(seed, dimensions)?);

    store.save(&existence)?;
    print(format!(
        "{name} born (seed {seed}, {dimensions} dimensions) in {}\n{}",
        directory.display(),
        ron::ser::to_string_pretty(
            existence.constitution().scales(),
            ron::ser::PrettyConfig::default()
        )?
    ))
}

fn parameters(line: &CommandLine) -> Result<LearningParameters, Box<dyn Error>> {
    let mut parameters = LearningParameters::default();

    if let Some(maximum) = line.number("--max-invocations")? {
        parameters = parameters.with_maximum_invocations(maximum);
    }

    if let Some(weight) = line.number("--cost-weight")? {
        parameters = parameters.with_cost_weight_per_second(weight);
    }

    if let Some(weight) = line.number("--scarcity-weight")? {
        parameters = parameters.with_scarcity_weight(weight);
    }

    if let Some(minimum) = line.number("--minimum-utility")? {
        parameters = parameters.with_minimum_utility(minimum);
    }

    // Measured durations then no longer weigh on decisions, so that a
    // virtual existence depends only on its seed and its stimuli.
    if line.flag("--deterministic") {
        parameters = parameters.with_cost_weight_per_second(0.0);
    }

    Ok(parameters)
}

fn live(line: &CommandLine) -> Result<(), Box<dyn Error>> {
    line.accept(
        &[
            "--for",
            "--clock",
            "--schedule",
            "--checkpoint",
            "--without",
            "--dormancy",
            "--cost-weight",
        ],
        &["--offline", "--without-influences", "--deterministic"],
    )?;

    let directory = individual(line)?;
    let (store, mut existence) = existing(&directory)?;
    let clock = match line.value("--clock").unwrap_or("real") {
        "real" => ClockMode::Real,
        "virtual" => ClockMode::Virtual,
        other => return Err(format!("Unknown clock {other}: real or virtual").into()),
    };
    let duration = line.number("--for")?.unwrap_or(f64::INFINITY);
    let schedule: Vec<ScheduledStimulus> = match line.value("--schedule") {
        Some(path) => ron::de::from_str(&fs::read_to_string(path)?)?,
        None => Vec::new(),
    };
    let excluded = line
        .all("--without")
        .iter()
        .map(|label| Activity::parse(label).ok_or(format!("Unknown activity {label}")))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let mut settings = LifeSettings::new(clock, duration, directory)
        .with_schedule(schedule)
        .without(excluded);

    if let Some(seconds) = line.number("--checkpoint")? {
        settings = settings.with_checkpoint(seconds);
    }

    if let Some(seconds) = line.number("--dormancy")? {
        settings = settings.with_dormancy(seconds);
    }

    if line.flag("--without-influences") {
        settings = settings.without_influences();
    }

    let intelligence = Intelligence::new(
        ModuleCatalog::standard(line.flag("--offline")),
        resources(line)?,
        parameters(line)?,
    );

    Life::new(&intelligence, settings).live(&mut existence, &store)?;
    print(LifeSummary::of(&existence))
}

fn episode(line: &CommandLine) -> Result<(), Box<dyn Error>> {
    line.accept(
        &[
            "--seed",
            "--max-invocations",
            "--cost-weight",
            "--scarcity-weight",
            "--minimum-utility",
            "--resource-budget",
        ],
        &[
            "--neutral",
            "--sandbox",
            "--offline",
            "--without-knowledge",
            "--without-routing",
            "--without-cache",
            "--without-influences",
            "--summary",
            "--detailed",
        ],
    )?;

    let (store, mut existence) = existing(&individual(line)?)?;
    let event: Event = ron::de::from_str(&read_stdin()?)?;
    let seed = match line.number("--seed")? {
        Some(seed) => seed,
        None => clock_seed()?,
    };
    let output = if line.flag("--detailed") {
        Output::Detailed
    } else if line.flag("--summary") {
        Output::Summary
    } else {
        Output::Report
    };
    let mut configuration = EpisodeConfiguration::new(
        seed,
        !line.flag("--without-knowledge"),
        !line.flag("--without-routing"),
    )
    .with_resource_budgets(line.pairs("--resource-budget")?.into_iter().collect());

    if line.flag("--without-influences") {
        configuration = configuration.without_influences();
    }

    if line.flag("--without-cache") {
        configuration = configuration.without_acquisition_cache();
    }

    let intelligence = Intelligence::new(
        ModuleCatalog::standard(line.flag("--offline")),
        resources(line)?,
        parameters(line)?,
    );

    existence.record(LifeEvent::Intervention {
        description: format!(
            "laboratory episode on {}{}{}",
            event.id(),
            if line.flag("--neutral") {
                ", neutral"
            } else {
                ""
            },
            if line.flag("--sandbox") {
                ", sandboxed"
            } else {
                ""
            }
        ),
    });

    let report = if line.flag("--neutral") {
        intelligence.reason_with(
            &event,
            existence.memory_mut(),
            configuration,
            output == Output::Detailed,
            &mut NeutralDisposition,
        )
    } else {
        let (mut organism, memory) = existence.organism(ClockMode::Real);

        intelligence.reason_with(
            &event,
            memory,
            configuration,
            output == Output::Detailed,
            &mut organism,
        )
    };

    if !line.flag("--sandbox") {
        store.save(&existence)?;
    }

    if output == Output::Summary {
        print(EpisodeSummary::new(
            existence.memory().experiences().episodes().len(),
            report.episode(),
        ))
    } else {
        print(ron::ser::to_string_pretty(
            &report,
            ron::ser::PrettyConfig::default(),
        )?)
    }
}

fn intervene(line: &CommandLine) -> Result<(), Box<dyn Error>> {
    line.accept(&["--latent", "--bias", "--scale", "--reason"], &[])?;

    let (store, mut existence) = existing(&individual(line)?)?;
    let mut changes = Vec::new();

    for (dimension, value) in line.pairs::<f64>("--latent")? {
        let index: usize = dimension
            .parse()
            .map_err(|_| format!("Invalid latent dimension {dimension}"))?;

        if index >= existence.latent().dimensions() || !(value > -1.0 && value < 1.0) {
            return Err(format!(
                "--latent {index}={value}: the dimension must exist and the value lie in (-1, 1)"
            )
            .into());
        }

        let mut pre = existence.latent().pre().to_vec();

        pre[index] = value.atanh();

        let fallback = existence.constitution().baseline().to_vec();

        existence.latent_mut().set_pre(pre, &fallback);
        changes.push(format!("latent {index} set to {value}"));
    }

    for (name, value) in line.pairs::<f64>("--bias")? {
        let channel = Channel::ALL
            .into_iter()
            .find(|channel| channel.label().eq_ignore_ascii_case(&name))
            .ok_or(format!("Unknown channel {name}"))?;

        if !value.is_finite() {
            return Err(format!("Invalid bias {value}").into());
        }

        existence.constitution_mut().set_bias(channel, value);
        changes.push(format!("bias of {channel} set to {value}"));
    }

    for (name, factor) in line.pairs::<f64>("--scale")? {
        let constitution = existence.constitution();
        let scales = constitution.scales().clone().scaled(&name, factor)?;
        let regenerated = crate::individuality::constitution::Constitution::from_scales(
            constitution.birth().clone(),
            scales,
        );

        *existence.constitution_mut() = regenerated;
        changes.push(format!(
            "scale {name} multiplied by {factor} (laws redrawn from the same seed)"
        ));
    }

    if changes.is_empty() {
        return Err(format!("Nothing to change\n{}", CommandLine::USAGE).into());
    }

    let description = format!(
        "{}{}",
        changes.join("; "),
        line.value("--reason")
            .map(|reason| format!(" ({reason})"))
            .unwrap_or_default()
    );

    existence.record(LifeEvent::Intervention {
        description: description.clone(),
    });
    store.save(&existence)?;
    print(description)
}
