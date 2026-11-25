use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{cargo_snapshot::CargoSnapshot, snapshot_identity::SnapshotIdentity};

#[derive(Debug, Default, Deserialize, Serialize)]
pub(crate) struct Knowledge {
    snapshots: BTreeMap<SnapshotIdentity, CargoSnapshot>,
}

impl Knowledge {
    pub(crate) fn integrate(&mut self, identity: SnapshotIdentity, snapshot: CargoSnapshot) {
        self.snapshots.entry(identity).or_insert(snapshot);
    }

    pub(crate) fn snapshot(&self, identity: &SnapshotIdentity) -> Option<&CargoSnapshot> {
        self.snapshots.get(identity)
    }
}
