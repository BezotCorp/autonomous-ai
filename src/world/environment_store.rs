use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use fs2::FileExt;

use crate::world::{environment::Environment, format_one::environment_v1::EnvironmentV1};

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
    /// 2: messages may carry rules. Format 1 is migrated when read.
    const FORMAT_VERSION: u32 = 2;

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

    /// Loads the environment. A file of an earlier format is migrated in
    /// memory; before it is ever overwritten, the original is kept beside
    /// it, unchanged (`environment.bin.v1`).
    pub(crate) fn load(&self) -> Result<Environment, Box<dyn Error>> {
        let (environment, version) = Self::decode(&self.path)?;

        if version != Self::FORMAT_VERSION {
            let kept = self.path.with_extension(format!("bin.v{version}"));

            if !kept.exists() {
                fs::copy(&self.path, &kept)?;
            }
        }

        Ok(environment)
    }

    /// Reads an environment without locking it (migrating it in memory if
    /// needed; nothing is written).
    pub(crate) fn read(path: &Path) -> Result<Environment, Box<dyn Error>> {
        Ok(Self::decode(path)?.0)
    }

    fn decode(path: &Path) -> Result<(Environment, u32), Box<dyn Error>> {
        let bytes =
            fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let payload = bytes
            .strip_prefix(Self::MAGIC.as_slice())
            .ok_or(format!("{} is not an environment file", path.display()))?;
        let (version, payload) = payload
            .split_at_checked(4)
            .ok_or("Truncated environment header")?;
        let version = u32::from_le_bytes(version.try_into()?);

        let configuration = bincode_next::config::standard();
        let (environment, consumed) = match version {
            1 => {
                let (old, consumed): (EnvironmentV1, usize) =
                    bincode_next::serde::decode_from_slice(payload, configuration)?;

                (old.migrate(), consumed)
            }
            Self::FORMAT_VERSION => {
                bincode_next::serde::decode_from_slice::<Environment, _>(payload, configuration)?
            }
            other => {
                return Err(format!(
                    "{} uses environment format {other}, this version reads formats 1 and {}",
                    path.display(),
                    Self::FORMAT_VERSION
                )
                .into());
            }
        };

        if consumed != payload.len() {
            return Err("Unexpected trailing environment data".into());
        }

        Ok((environment, version))
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

#[cfg(test)]
mod tests {
    use super::EnvironmentStore;
    use crate::{
        representation::statement::Statement,
        world::{member::Member, transmission_status::TransmissionStatus},
    };

    #[test]
    fn migrates_format_one_without_touching_the_original() {
        let directory = tempfile::tempdir().unwrap();
        let statement: Statement = "colour(sky, blue)".parse().unwrap();
        // Field by field, the layout format 1 wrote.
        let transmission = (
            uuid::Uuid::now_v7(),
            "e1".to_owned(),
            "e2".to_owned(),
            10.0_f64,
            70.0_f64,
            (
                vec![statement.clone()],
                Vec::<(Statement, String)>::new(),
                Vec::<Statement>::new(),
                Vec::<crate::representation::document::Document>::new(),
            ),
            TransmissionStatus::Pending,
        );
        let environment = (
            uuid::Uuid::now_v7(),
            "old".to_owned(),
            7_u64,
            chrono::Utc::now(),
            120.0_f64,
            60.0_f64,
            1.0_f64,
            false,
            vec![
                Member::new("e1".to_owned(), "a".to_owned(), 0.0),
                Member::new("e2".to_owned(), "b".to_owned(), 0.0),
            ],
            vec![transmission],
        );
        let mut bytes = b"AAIE".to_vec();

        bytes.extend_from_slice(&1_u32.to_le_bytes());
        bytes.extend(
            bincode_next::serde::encode_to_vec(&environment, bincode_next::config::standard())
                .unwrap(),
        );
        std::fs::write(EnvironmentStore::path(directory.path()), &bytes).unwrap();

        let store = EnvironmentStore::open(directory.path()).unwrap();
        let migrated = store.load().unwrap();

        assert_eq!(migrated.name(), "old");
        assert_eq!(migrated.clock(), 120.0);
        assert_eq!(migrated.members().len(), 2);
        assert_eq!(
            migrated.transmissions()[0].message().statements(),
            &[statement]
        );
        assert!(migrated.transmissions()[0].message().rules().is_empty());
        assert_eq!(
            std::fs::read(directory.path().join("environment.bin.v1")).unwrap(),
            bytes
        );

        store.save(&migrated).unwrap();
        assert_eq!(store.load().unwrap().transmissions().len(), 1);
        assert_eq!(
            std::fs::read(directory.path().join("environment.bin.v1")).unwrap(),
            bytes
        );
    }
}
