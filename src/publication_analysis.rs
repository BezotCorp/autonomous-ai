use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{older_resolved_package::OlderResolvedPackage, snapshot_identity::SnapshotIdentity};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct PublicationAnalysis {
    id: Uuid,
    event_id: String,
    analyzed_at: DateTime<Utc>,
    examined_snapshots: Vec<SnapshotIdentity>,
    older_packages: Vec<OlderResolvedPackage>,
}

impl PublicationAnalysis {
    pub(crate) fn new(
        event_id: String,
        examined_snapshots: Vec<SnapshotIdentity>,
        older_packages: Vec<OlderResolvedPackage>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            event_id,
            analyzed_at: Utc::now(),
            examined_snapshots,
            older_packages,
        }
    }
}
