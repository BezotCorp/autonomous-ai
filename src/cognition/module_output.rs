use crate::{
    cognition::proposal::Proposal,
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
    corrections: Vec<(Correction, CorrectionBasis, String)>,
    experiments: Vec<ExperimentRequest>,
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
    /// declaring `reflective` may do so.
    pub(crate) fn correct(
        &mut self,
        correction: Correction,
        basis: CorrectionBasis,
        justification: impl Into<String>,
    ) {
        self.corrections
            .push((correction, basis, justification.into()));
    }

    /// Requests an internal experiment; only reflective modules may do so.
    pub(crate) fn experiment(&mut self, request: ExperimentRequest) {
        self.experiments.push(request);
    }

    pub(crate) fn into_parts(self) -> (Vec<Proposal>, Vec<String>, Option<Plan>) {
        (self.proposals, self.notes, self.plan)
    }

    /// Splits off the corrections and experiments, leaving the rest.
    pub(crate) fn take_reflections(
        &mut self,
    ) -> (
        Vec<(Correction, CorrectionBasis, String)>,
        Vec<ExperimentRequest>,
    ) {
        (
            std::mem::take(&mut self.corrections),
            std::mem::take(&mut self.experiments),
        )
    }
}
