use uuid::Uuid;

use crate::{
    cognition::{proposal::Proposal, reflections::Reflections},
    influences::influence::Influence,
    memory::{
        correction::Correction, correction_basis::CorrectionBasis,
        experiment_request::ExperimentRequest,
    },
    planning::plan::Plan,
};

/// Everything a module returns from one invocation.
#[derive(Debug, Default)]
pub(crate) struct ModuleOutput {
    proposals: Vec<Proposal>,
    notes: Vec<String>,
    plan: Option<Plan>,
    reflections: Reflections,
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

    /// Proposes a justified correction of the memory; only modules
    /// declaring `reflective` may do so (as for what follows).
    pub(crate) fn correct(
        &mut self,
        correction: Correction,
        basis: CorrectionBasis,
        justification: impl Into<String>,
    ) {
        self.reflections
            .correct(correction, basis, justification.into());
    }

    /// Requests an internal experiment.
    pub(crate) fn experiment(&mut self, request: ExperimentRequest) {
        self.reflections.experiment(request);
    }

    /// Proposes to influence the context of the conscious.
    pub(crate) fn influence(&mut self, influence: Influence) {
        self.reflections.influence(influence);
    }

    /// Reports a dream as examined.
    pub(crate) fn examined(&mut self, dream: Uuid) {
        self.reflections.examine(dream);
    }

    pub(crate) fn into_parts(self) -> (Vec<Proposal>, Vec<String>, Option<Plan>) {
        (self.proposals, self.notes, self.plan)
    }

    pub(crate) fn take_reflections(&mut self) -> Reflections {
        std::mem::take(&mut self.reflections)
    }
}
