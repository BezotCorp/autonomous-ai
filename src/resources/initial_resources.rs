use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;

use crate::{
    acquisition::documentation_query::DocumentationQuery,
    resources::{initial_knowledge::InitialKnowledge, lexicon::Lexicon},
};

/// Resources available from the start, loaded from a directory. Each file
/// is optional; a missing file means the resource is absent, while an
/// unreadable or malformed file is an error.
#[derive(Debug, Default)]
pub(crate) struct InitialResources {
    knowledge: InitialKnowledge,
    documentation_queries: Vec<DocumentationQuery>,
    lexicon: Lexicon,
}

impl InitialResources {
    pub(crate) const KNOWLEDGE_FILE: &'static str = "knowledge.ron";
    const DOCUMENTATION_QUERIES_FILE: &'static str = "documentation-queries.ron";
    const LEXICON_FILE: &'static str = "lexicon.ron";

    pub(crate) fn load(directory: &Path) -> Result<Self, Box<dyn Error>> {
        if !directory.is_dir() {
            return Err(format!("Resource directory not found: {}", directory.display()).into());
        }

        Ok(Self {
            knowledge: Self::read(directory.join(Self::KNOWLEDGE_FILE))?.unwrap_or_default(),
            documentation_queries: Self::read(directory.join(Self::DOCUMENTATION_QUERIES_FILE))?
                .unwrap_or_default(),
            lexicon: Self::read::<Lexicon>(directory.join(Self::LEXICON_FILE))?
                .unwrap_or_default()
                .index(),
        })
    }

    fn read<T: DeserializeOwned>(path: PathBuf) -> Result<Option<T>, Box<dyn Error>> {
        match fs::read_to_string(&path) {
            Ok(content) => ron::de::from_str(&content)
                .map(Some)
                .map_err(|error| format!("{}: {error}", path.display()).into()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!("{}: {error}", path.display()).into()),
        }
    }

    pub(crate) fn knowledge(&self) -> &InitialKnowledge {
        &self.knowledge
    }

    pub(crate) fn documentation_queries(&self) -> &[DocumentationQuery] {
        &self.documentation_queries
    }

    pub(crate) fn lexicon(&self) -> &Lexicon {
        &self.lexicon
    }
}
