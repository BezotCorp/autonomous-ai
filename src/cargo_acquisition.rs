use crate::{cargo_snapshot::CargoSnapshot, memory::Memory, snapshot_identity::SnapshotIdentity};

pub(crate) struct CargoAcquisition {
    identity: SnapshotIdentity,
    snapshot: Option<CargoSnapshot>,
}

impl CargoAcquisition {
    pub(crate) fn new(identity: SnapshotIdentity, snapshot: Option<CargoSnapshot>) -> Self {
        Self { identity, snapshot }
    }

    pub(crate) fn integrate(self, memory: &mut Memory) -> (SnapshotIdentity, bool) {
        let reused = self.snapshot.is_none();

        if let Some(snapshot) = self.snapshot {
            memory.integrate_snapshot(self.identity.clone(), snapshot);
        }

        (self.identity, reused)
    }
}
