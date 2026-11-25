use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Capability {
    DiscoverRepositories,
    ObserveRepository,
}
