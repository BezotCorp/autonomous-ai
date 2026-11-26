use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use fs2::FileExt;

use crate::world::environment::Environment;

/// Storage of an environment in its directory (`environment.bin`), apart
/// from the existences, which keep their own files: an existence never
/// depends on its environment to be read, saved or moved elsewhere.
pub(crate) struct EnvironmentStore {
    path: PathBuf,
    _lock: File,
}

impl EnvironmentStore {
    const FILE: &'static str = "environment.bin";
    const MAGIC: &'static [u8; 4] = b"AAIE";
    const FORMAT_VERSION: u32 = 1;

    pub(crate) fn path(directory: &Path) -> PathBuf {
        directory.join(Self::FILE)
    }

    pub(crate) fn open(directory: &Path) -> Result<Self, Box<dyn Error>> {
        fs::create_dir_all(directory)?;

        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(directory.join(format!("{}.lock", Self::FILE)))?;

        lock.try_lock_exclusive()
            .map_err(|_| format!("{} is in use by another process", directory.display()))?;

        Ok(Self {
            path: Self::path(directory),
            _lock: lock,
        })
    }

    pub(crate) fn exists(&self) -> bool {
        self.path.is_file()
    }

    pub(crate) fn load(&self) -> Result<Environment, Box<dyn Error>> {
        Self::read(&self.path)
    }

    pub(crate) fn read(path: &Path) -> Result<Environment, Box<dyn Error>> {
        let bytes =
            fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let payload = bytes
            .strip_prefix(Self::MAGIC.as_slice())
            .ok_or(format!("{} is not an environment file", path.display()))?;
        let (version, payload) = payload
            .split_at_checked(4)
            .ok_or("Truncated environment header")?;
        let version = u32::from_le_bytes(version.try_into()?);

        if version != Self::FORMAT_VERSION {
            return Err(format!(
                "{} uses environment format {version}, this version reads format {}",
                path.display(),
                Self::FORMAT_VERSION
            )
            .into());
        }

        let (environment, consumed): (Environment, usize) =
            bincode_next::serde::decode_from_slice(payload, bincode_next::config::standard())?;

        if consumed != payload.len() {
            return Err("Unexpected trailing environment data".into());
        }

        Ok(environment)
    }

    pub(crate) fn save(&self, environment: &Environment) -> Result<(), Box<dyn Error>> {
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        let mut bytes = Self::MAGIC.to_vec();

        bytes.extend_from_slice(&Self::FORMAT_VERSION.to_le_bytes());
        bytes.extend(bincode_next::serde::encode_to_vec(
            environment,
            bincode_next::config::standard(),
        )?);

        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;

        temporary.write_all(&bytes)?;
        temporary.flush()?;
        temporary.as_file().sync_all()?;
        temporary.persist(&self.path)?;

        #[cfg(unix)]
        File::open(parent)?.sync_all()?;

        Ok(())
    }
}
