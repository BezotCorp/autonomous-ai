use crate::cognition::proposal::Proposal;

/// Everything a module returns from one invocation.
#[derive(Debug, Default)]
pub(crate) struct ModuleOutput {
    proposals: Vec<Proposal>,
    notes: Vec<String>,
}

impl ModuleOutput {
    pub(crate) fn propose(&mut self, proposal: Proposal) {
        self.proposals.push(proposal);
    }

    /// Records an observable remark about what the module did or could not do.
    pub(crate) fn note(&mut self, note: impl Into<String>) {
        self.notes.push(note.into());
    }

    pub(crate) fn into_parts(self) -> (Vec<Proposal>, Vec<String>) {
        (self.proposals, self.notes)
    }
}
