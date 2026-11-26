use serde::{Deserialize, Serialize};

use crate::{reasoning::epistemic_kind::EpistemicKind, representation::content::Content};

/// Element of an imagined setting, with how it came to be there:
/// recalled from memory (`Recollection`), reproduced from a recorded
/// acquisition (`Replay`) or invented or transformed (`Imagined`), and by
/// which mechanism from what (`origin`).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct ImaginedElement {
    content: Content,
    kind: EpistemicKind,
    origin: String,
}

impl ImaginedElement {
    pub(crate) fn new(content: Content, kind: EpistemicKind, origin: String) -> Self {
        Self {
            content,
            kind,
            origin,
        }
    }

    pub(crate) fn content(&self) -> &Content {
        &self.content
    }

    pub(crate) fn kind(&self) -> EpistemicKind {
        self.kind
    }

    pub(crate) fn origin(&self) -> &str {
        &self.origin
    }
}
