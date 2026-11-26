use serde::{Deserialize, Serialize};

/// One meaning of a lexical entry, with its semantic relations
/// (synonymy, antonymy, hypernymy...) as named by the source resource.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Sense {
    identifier: String,
    definition: String,
    #[serde(default)]
    relations: Vec<(String, String)>,
}

impl Sense {
    pub(crate) fn identifier(&self) -> &str {
        &self.identifier
    }

    pub(crate) fn definition(&self) -> &str {
        &self.definition
    }

    pub(crate) fn relations(&self) -> &[(String, String)] {
        &self.relations
    }
}
