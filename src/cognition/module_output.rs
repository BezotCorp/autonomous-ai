use crate::{cognition::proposal::Proposal, planning::plan::Plan};

/// Everything a module returns from one invocation.
#[derive(Debug, Default)]
pub(crate) struct ModuleOutput {
    proposals: Vec<Proposal>,
    notes: Vec<String>,
    plan: Option<Plan>,
}

impl ModuleOutput {
    pub(crate) fn propose(&mut self, proposal: Proposal) {
        self.proposals.push(proposal);
    }

    /// Records an observable remark about what the module did or could not do.
    pub(crate) fn note(&mut self, note: impl Into<String>) {
        self.notes.push(note.into());
    }

    /// Proposes a plan; only modules declaring `planning` may do so.
    pub(crate) fn plan(&mut self, plan: Plan) {
        self.plan = Some(plan);
    }

    pub(crate) fn into_parts(self) -> (Vec<Proposal>, Vec<String>, Option<Plan>) {
        (self.proposals, self.notes, self.plan)
    }
}
