use serde::{Deserialize, Serialize};

use crate::memory::{dream_record::DreamRecord, experiment_request::ExperimentRequest};

/// Register of dreams, kept apart from real experiences, and of the
/// experiments still to be dreamt.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct DreamMemory {
    dreams: Vec<DreamRecord>,
    pending: Vec<ExperimentRequest>,
}

impl DreamMemory {
    /// Dreams whose detailed trace is kept; older ones keep their summary.
    const DETAILED_DREAMS: usize = 50;

    pub(crate) fn dreams(&self) -> &[DreamRecord] {
        &self.dreams
    }

    pub(crate) fn pending(&self) -> &[ExperimentRequest] {
        &self.pending
    }

    /// Queues an experiment unless the same one is queued or was dreamt.
    pub(crate) fn request(&mut self, request: ExperimentRequest) -> bool {
        let known = self.pending.iter().any(|pending| pending.same_as(&request))
            || self.dreams.iter().any(|dream| {
                dream.original() == request.episode()
                    && dream.mode() == request.mode()
                    && dream.variant() == request.variant()
            });

        if !known {
            self.pending.push(request);
        }

        !known
    }

    pub(crate) fn take(&mut self, index: usize) -> Option<ExperimentRequest> {
        (index < self.pending.len()).then(|| self.pending.remove(index))
    }

    pub(crate) fn record(&mut self, dream: DreamRecord) {
        self.dreams.push(dream);

        let detailed = self.dreams.len().saturating_sub(Self::DETAILED_DREAMS);

        for dream in &mut self.dreams[..detailed] {
            dream.forget_trace();
        }
    }
}
