use std::{
    error::Error,
    io::{self, Write},
    process::ExitCode,
};

use serde::Serialize;

use crate::{
    arguments::Arguments, event::Event, intelligence::Intelligence, memory_store::MemoryStore,
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
        Arguments::Receive(path) => {
            let event: Event = ron::de::from_reader(io::stdin().lock())?;

            let store = MemoryStore::open(&path)?;

            let mut intelligence = Intelligence::new(store.load()?);

            let report = intelligence.receive(event, &store)?;

            write_output(&report)?;
        }
        Arguments::Inspect(path) => {
            if !path.is_file() {
                return Err(format!("Memory file does not exist: {}", path.display(),).into());
            }

            let store = MemoryStore::open(&path)?;
            let memory = store.load()?;

            write_output(&memory)?;
        }
    }

    Ok(())
}

fn write_output<T: Serialize>(value: &T) -> Result<(), Box<dyn Error>> {
    let output = ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::default())?;

    let mut stdout = io::stdout().lock();

    writeln!(stdout, "{output}")?;

    Ok(())
}
