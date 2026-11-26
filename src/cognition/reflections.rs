use uuid::Uuid;

use crate::{
    influences::influence::Influence,
    memory::{
        correction::Correction, correction_basis::CorrectionBasis,
        experiment_request::ExperimentRequest,
    },
};

/// What a reflective module proposes beyond reasoning contents: justified
/// corrections of the memory, internal experiments, influences on the
/// conscious, and the dreams it examined.
#[derive(Debug, Default)]
pub(crate) struct Reflections {
    corrections: Vec<(Correction, CorrectionBasis, String)>,
    experiments: Vec<ExperimentRequest>,
    influences: Vec<Influence>,
    examined: Vec<Uuid>,
}

impl Reflections {
    pub(crate) fn correct(
        &mut self,
        correction: Correction,
        basis: CorrectionBasis,
        justification: String,
    ) {
        self.corrections.push((correction, basis, justification));
    }

    pub(crate) fn experiment(&mut self, request: ExperimentRequest) {
        self.experiments.push(request);
    }

    pub(crate) fn influence(&mut self, influence: Influence) {
        self.influences.push(influence);
    }

    pub(crate) fn examine(&mut self, dream: Uuid) {
        self.examined.push(dream);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.corrections.is_empty()
            && self.experiments.is_empty()
            && self.influences.is_empty()
            && self.examined.is_empty()
    }

    pub(crate) fn corrections(&self) -> &[(Correction, CorrectionBasis, String)] {
        &self.corrections
    }

    pub(crate) fn extend(&mut self, other: Self) {
        self.corrections.extend(other.corrections);
        self.experiments.extend(other.experiments);
        self.influences.extend(other.influences);
        self.examined.extend(other.examined);
    }

    #[allow(clippy::type_complexity)]
    pub(crate) fn into_parts(
        self,
    ) -> (
        Vec<(Correction, CorrectionBasis, String)>,
        Vec<ExperimentRequest>,
        Vec<Influence>,
        Vec<Uuid>,
    ) {
        (
            self.corrections,
            self.experiments,
            self.influences,
            self.examined,
        )
    }
}
