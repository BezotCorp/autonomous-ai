use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use fs2::FileExt;

use crate::memory::Memory;

const MAGIC: &[u8; 4] = b"AAIM";
const SCHEMA_VERSION: u32 = 1;

pub(crate) struct MemoryStore {
    path: PathBuf,
    lock: File,
}

impl MemoryStore {
    pub(crate) fn open(path: &Path) -> Result<Self, Box<dyn Error>> {
        let parent = Self::parent(path);

        fs::create_dir_all(parent)?;

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
            lock,
        })
    }

    pub(crate) fn load(&self) -> Result<Memory, Box<dyn Error>> {
        let mut file = match File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(Memory::default());
            }
            Err(error) => return Err(error.into()),
        };

        let mut magic = [0_u8; 4];
        file.read_exact(&mut magic)?;

        if &magic != MAGIC {
            return Err("Invalid memory file signature".into());
        }

        let mut version = [0_u8; 4];
        file.read_exact(&mut version)?;

        let version = u32::from_le_bytes(version);

        if version != SCHEMA_VERSION {
            return Err(format!("Unsupported memory schema version: {version}").into());
        }

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        let config = bincode_next::config::standard();

        let (memory, consumed): (Memory, usize) =
            bincode_next::serde::decode_from_slice(&bytes, config)?;

        if consumed != bytes.len() {
            return Err("Unexpected trailing memory data".into());
        }

        Ok(memory)
    }

    pub(crate) fn save(&self, memory: &Memory) -> Result<(), Box<dyn Error>> {
        let parent = Self::parent(&self.path);
        let config = bincode_next::config::standard();

        let bytes = bincode_next::serde::encode_to_vec(memory, config)?;

        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;

        temporary.write_all(MAGIC)?;
        temporary.write_all(&SCHEMA_VERSION.to_le_bytes())?;
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

impl Drop for MemoryStore {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.lock);
    }
}
