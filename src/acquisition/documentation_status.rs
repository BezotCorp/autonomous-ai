use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DocumentationStatus {
    Available,
    Partial,
    NotFound,
    Unavailable,
    Unsupported,
}
