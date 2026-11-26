use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;

use crate::events::event::Event;

/// Exchanges of an individual with the outside, through its directory:
/// stimuli are files dropped in `inbox/` (taken in name order, then moved
/// to `inbox/received/`, or to `inbox/rejected/` when unreadable); what the
/// individual answers is written to `outbox/`. A file named `stop` asks
/// the process to stop (a technical stop, not a cognitive decision).
pub(crate) struct Mailbox {
    directory: PathBuf,
}

impl Mailbox {
    pub(crate) fn new(directory: &Path) -> Self {
        Self {
            directory: directory.to_path_buf(),
        }
    }

    fn inbox(&self) -> PathBuf {
        self.directory.join("inbox")
    }

    /// Drops an event in the inbox (used by `stimulate`).
    pub(crate) fn deliver(&self, event: &Event, raw: &str) -> Result<PathBuf, Box<dyn Error>> {
        let inbox = self.inbox();

        fs::create_dir_all(&inbox)?;

        let path = inbox.join(format!(
            "{}-{}.ron",
            chrono::Utc::now().format("%Y%m%dT%H%M%S%.6f"),
            event
                .id()
                .chars()
                .map(|character| if character.is_ascii_alphanumeric() {
                    character
                } else {
                    '_'
                })
                .collect::<String>()
        ));

        fs::write(&path, raw)?;
        Ok(path)
    }

    /// Takes the events waiting in the inbox; unreadable files are set
    /// aside and reported.
    pub(crate) fn collect(&self) -> (Vec<Event>, Vec<String>) {
        let inbox = self.inbox();
        let mut events = Vec::new();
        let mut errors = Vec::new();
        let Ok(entries) = fs::read_dir(&inbox) else {
            return (events, errors);
        };
        let mut files: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "ron"))
            .collect();

        files.sort();

        for path in files {
            let parsed = fs::read_to_string(&path)
                .map_err(|error| error.to_string())
                .and_then(|text| {
                    ron::de::from_str::<Event>(&text).map_err(|error| error.to_string())
                });
            let destination = match &parsed {
                Ok(_) => inbox.join("received"),
                Err(_) => inbox.join("rejected"),
            };
            let moved = fs::create_dir_all(&destination).and_then(|()| {
                fs::rename(
                    &path,
                    destination.join(path.file_name().unwrap_or_default()),
                )
            });

            match (parsed, moved) {
                (Ok(event), Ok(())) => events.push(event),
                (Ok(_), Err(error)) => errors.push(format!("{}: {error}", path.display())),
                (Err(error), _) => errors.push(format!("{}: {error}", path.display())),
            }
        }

        (events, errors)
    }

    pub(crate) fn respond<T: Serialize>(
        &self,
        name: &str,
        response: &T,
    ) -> Result<(), Box<dyn Error>> {
        let outbox = self.directory.join("outbox");

        fs::create_dir_all(&outbox)?;
        fs::write(
            outbox.join(format!("{name}.ron")),
            ron::ser::to_string_pretty(response, ron::ser::PrettyConfig::default())?,
        )?;
        Ok(())
    }

    pub(crate) fn stop_requested(&self) -> bool {
        self.directory.join("stop").exists()
    }

    pub(crate) fn request_stop(&self) -> Result<(), Box<dyn Error>> {
        fs::write(self.directory.join("stop"), "")?;
        Ok(())
    }

    pub(crate) fn clear_stop(&self) -> std::io::Result<()> {
        match fs::remove_file(self.directory.join("stop")) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        }
    }
}
