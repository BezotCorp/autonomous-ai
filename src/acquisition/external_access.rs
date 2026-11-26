use crate::{
    acquisition::response_store::ResponseStore, consumables::resource_meter::ResourceMeter,
    memory::acquisition_cache::AcquisitionCache,
};

/// Everything acquisition code needs to reach external sources sparingly:
/// the meter of consumable resources and the store of acquired data.
#[derive(Debug)]
pub(crate) struct ExternalAccess<'a> {
    meter: ResourceMeter,
    store: ResponseStore<'a>,
}

impl<'a> ExternalAccess<'a> {
    pub(crate) fn new(meter: ResourceMeter, cache: &'a AcquisitionCache) -> Self {
        Self {
            meter,
            store: ResponseStore::new(cache),
        }
    }

    pub(crate) fn meter(&self) -> &ResourceMeter {
        &self.meter
    }

    pub(crate) fn store(&self) -> &ResponseStore<'a> {
        &self.store
    }

    pub(crate) fn into_parts(self) -> (ResourceMeter, ResponseStore<'a>) {
        (self.meter, self.store)
    }
}
