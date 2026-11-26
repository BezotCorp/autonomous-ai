use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use fs2::FileExt;

use crate::living::existence::Existence;

/// Storage of an existence in its directory (`existence.bin`). A living or
/// modifying process holds an exclusive lock; observers read the last
/// saved state without locking, which never disturbs the individual
/// (saves replace the file atomically).
pub(crate) struct ExistenceStore {
    path: PathBuf,
    _lock: File,
}

impl ExistenceStore {
    const FILE: &'static str = "existence.bin";
    /// Identifies existence files and the version of their layout.
    const MAGIC: &'static [u8; 4] = b"AAIX";
    const FORMAT_VERSION: u32 = 1;

    pub(crate) fn path(directory: &Path) -> PathBuf {
        directory.join(Self::FILE)
    }

    /// Opens the store of an individual for modification.
    pub(crate) fn open(directory: &Path) -> Result<Self, Box<dyn Error>> {
        fs::create_dir_all(directory)?;

        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(directory.join(format!("{}.lock", Self::FILE)))?;

        lock.try_lock_exclusive().map_err(|_| {
            format!(
                "{} is in use by another process (a living session?)",
                directory.display()
            )
        })?;

        Ok(Self {
            path: Self::path(directory),
            _lock: lock,
        })
    }

    pub(crate) fn exists(&self) -> bool {
        self.path.is_file()
    }

    pub(crate) fn load(&self) -> Result<Existence, Box<dyn Error>> {
        Self::read(&self.path)
    }

    /// Reads a saved existence without locking it.
    pub(crate) fn read(path: &Path) -> Result<Existence, Box<dyn Error>> {
        let bytes =
            fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let payload = bytes.strip_prefix(Self::MAGIC.as_slice()).ok_or(format!(
            "{} is not an existence file (earlier memory formats are not read; \
             they are left untouched)",
            path.display()
        ))?;
        let (version, payload) = payload
            .split_at_checked(4)
            .ok_or("Truncated existence header")?;
        let version = u32::from_le_bytes(version.try_into()?);

        if version != Self::FORMAT_VERSION {
            return Err(format!(
                "{} uses existence format {version}, this version reads format {}",
                path.display(),
                Self::FORMAT_VERSION
            )
            .into());
        }

        let (existence, consumed): (Existence, usize) =
            bincode_next::serde::decode_from_slice(payload, bincode_next::config::standard())?;

        if consumed != payload.len() {
            return Err("Unexpected trailing existence data".into());
        }

        Ok(existence)
    }

    /// The bytes of the file an existence is saved as.
    pub(crate) fn encode(existence: &Existence) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut bytes = Self::MAGIC.to_vec();

        bytes.extend_from_slice(&Self::FORMAT_VERSION.to_le_bytes());
        bytes.extend(bincode_next::serde::encode_to_vec(
            existence,
            bincode_next::config::standard(),
        )?);
        Ok(bytes)
    }

    pub(crate) fn save(&self, existence: &Existence) -> Result<(), Box<dyn Error>> {
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        let bytes = Self::encode(existence)?;

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
