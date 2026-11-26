use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use fs2::FileExt;

use crate::memory::persistent_memory::PersistentMemory;

pub(crate) struct MemoryStore {
    path: PathBuf,
    _lock: File,
}

impl MemoryStore {
    /// Identifies memory files and the version of their layout, so that a
    /// file written by an incompatible version is reported as such.
    const MAGIC: &'static [u8; 4] = b"AAIM";
    const FORMAT_VERSION: u32 = 3;

    pub(crate) fn open(path: &Path) -> Result<Self, Box<dyn Error>> {
        fs::create_dir_all(Self::parent(path))?;

        let mut lock_path = path.as_os_str().to_os_string();
        lock_path.push(".lock");

        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)?;

        FileExt::lock_exclusive(&lock)?;

        Ok(Self {
            path: path.to_path_buf(),
            _lock: lock,
        })
    }

    pub(crate) fn load(&self) -> Result<PersistentMemory, Box<dyn Error>> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,

            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(PersistentMemory::default());
            }

            Err(error) => return Err(error.into()),
        };

        let Some(payload) = bytes.strip_prefix(Self::MAGIC.as_slice()) else {
            return Err(format!(
                "{} was written by an earlier, incompatible version of the memory format; \
                 move it aside and let a new memory be created",
                self.path.display()
            )
            .into());
        };

        let (version, payload) = payload
            .split_at_checked(4)
            .ok_or("Truncated memory header")?;
        let version = u32::from_le_bytes(version.try_into()?);

        if version != Self::FORMAT_VERSION {
            return Err(format!(
                "{} uses memory format {version}, this version reads format {}",
                self.path.display(),
                Self::FORMAT_VERSION
            )
            .into());
        }

        let (memory, consumed): (PersistentMemory, usize) =
            bincode_next::serde::decode_from_slice(payload, bincode_next::config::standard())?;

        if consumed != payload.len() {
            return Err("Unexpected trailing memory data".into());
        }

        Ok(memory)
    }

    pub(crate) fn save(&self, memory: &PersistentMemory) -> Result<(), Box<dyn Error>> {
        let parent = Self::parent(&self.path);

        let mut bytes = Self::MAGIC.to_vec();

        bytes.extend_from_slice(&Self::FORMAT_VERSION.to_le_bytes());
        bytes.extend(bincode_next::serde::encode_to_vec(
            memory,
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

    fn parent(path: &Path) -> &Path {
        path.parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
    }
}
