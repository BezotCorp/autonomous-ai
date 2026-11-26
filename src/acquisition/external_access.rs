use chrono::{DateTime, Utc};

use crate::{
    acquisition::{access_mode::AccessMode, response_store::ResponseStore},
    consumables::resource_meter::ResourceMeter,
    memory::acquisition_cache::AcquisitionCache,
};

/// Everything acquisition code needs to reach external sources sparingly:
/// the meter of consumable resources, the store of acquired data and the
/// mode of access. In replay mode no request may leave the process.
#[derive(Debug)]
pub(crate) struct ExternalAccess<'a> {
    meter: ResourceMeter,
    store: ResponseStore<'a>,
    mode: AccessMode,
}

impl<'a> ExternalAccess<'a> {
    pub(crate) fn new(meter: ResourceMeter, cache: &'a AcquisitionCache) -> Self {
        Self {
            meter,
            store: ResponseStore::new(cache),
            mode: AccessMode::Live,
        }
    }

    /// Access serving only what was recorded by `as_of`.
    pub(crate) fn replay(
        meter: ResourceMeter,
        cache: &'a AcquisitionCache,
        as_of: DateTime<Utc>,
        since: DateTime<Utc>,
    ) -> Self {
        Self {
            meter,
            store: ResponseStore::new(cache),
            mode: AccessMode::Replay { as_of, since },
        }
    }

    pub(crate) fn meter(&self) -> &ResourceMeter {
        &self.meter
    }

    pub(crate) fn store(&self) -> &ResponseStore<'a> {
        &self.store
    }

    pub(crate) fn mode(&self) -> AccessMode {
        self.mode
    }

    pub(crate) fn is_live(&self) -> bool {
        self.mode == AccessMode::Live
    }

    pub(crate) fn into_parts(self) -> (ResourceMeter, ResponseStore<'a>) {
        (self.meter, self.store)
    }
}
