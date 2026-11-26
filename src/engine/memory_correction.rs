use uuid::Uuid;

use crate::memory::{
    correction::Correction, correction_basis::CorrectionBasis, correction_record::CorrectionRecord,
    hypothesis_status::HypothesisStatus, persistent_memory::PersistentMemory,
    previous_state::PreviousState,
};

/// Applies the corrections proposed by the examination of the cognition,
/// journaling what each replaced so that it stays explainable and can be
/// reverted. A correction that no longer applies is refused, not forced.
pub(crate) struct MemoryCorrection;

impl MemoryCorrection {
    pub(crate) fn apply(
        memory: &mut PersistentMemory,
        episode: Uuid,
        correction: Correction,
        basis: CorrectionBasis,
        justification: String,
    ) -> Result<Uuid, String> {
        // Reopening what a correction refuted is reverting that correction.
        if let Correction::ReopenHypothesis { rule } = &correction
            && let Some(refutation) = memory
                .corrections()
                .records()
                .iter()
                .find(|record| {
                    record.is_active()
                        && record.correction()
                            == &Correction::RefuteHypothesis { rule: rule.clone() }
                })
                .map(CorrectionRecord::id)
        {
            Self::revert(memory, refutation, justification)?;
            return Ok(refutation);
        }

        if memory.corrections().covers(&correction.subject()) {
            return Err(format!("{correction}: already corrected"));
        }

        let previous = match &correction {
            Correction::RefuteHypothesis { rule } => {
                let previous = memory.knowledge().hypothesis(rule).cloned();

                if previous.as_ref().map(|record| record.status()) != Some(&HypothesisStatus::Open)
                {
                    return Err(format!("{correction}: no open hypothesis"));
                }

                let (knowledge, _, _) = memory.parts_mut();

                knowledge.refute(rule, justification.clone());
                PreviousState::Hypothesis(previous)
            }
            Correction::ReopenHypothesis { rule } => {
                let previous = memory.knowledge().hypothesis(rule).cloned();

                if previous.as_ref().map(|record| record.status())
                    != Some(&HypothesisStatus::Refuted)
                {
                    return Err(format!("{correction}: no refuted hypothesis"));
                }

                let (knowledge, _, _) = memory.parts_mut();

                if let Some(record) = knowledge.hypothesis_mut(rule) {
                    record.reopen();
                }

                PreviousState::Hypothesis(previous)
            }
            Correction::ObsoleteFact { statement } => {
                let previous = memory.knowledge().entry(statement).cloned();
                let (knowledge, _, _) = memory.parts_mut();

                if !knowledge.withdraw(statement, justification.clone()) {
                    return Err(format!("{correction}: not an active belief"));
                }

                PreviousState::Fact(previous)
            }
            Correction::ReassessInvocation {
                episode: past,
                step,
                outcome,
                ..
            } => {
                if memory.experiences().find(*past).is_none() {
                    return Err(format!("{correction}: unknown episode"));
                }

                let previous = memory
                    .corrections_mut()
                    .reassess(*past, *step, Some(*outcome));

                PreviousState::Outcome(previous)
            }
        };

        let record = CorrectionRecord::new(episode, correction, basis, justification, previous);
        let id = record.id();

        memory.corrections_mut().record(record);

        if matches!(
            memory
                .corrections()
                .records()
                .last()
                .map(CorrectionRecord::correction),
            Some(Correction::ReassessInvocation { .. })
        ) {
            memory.relearn_routing();
        }

        Ok(id)
    }

    /// Restores what a correction replaced.
    pub(crate) fn revert(
        memory: &mut PersistentMemory,
        id: Uuid,
        reason: String,
    ) -> Result<(), String> {
        let record = memory
            .corrections()
            .records()
            .iter()
            .find(|record| record.id() == id)
            .cloned()
            .ok_or_else(|| format!("unknown correction {id}"))?;

        if !record.is_active() {
            return Err(format!("correction {id} was already reverted"));
        }

        match (record.correction(), record.previous().clone()) {
            (
                Correction::RefuteHypothesis { rule } | Correction::ReopenHypothesis { rule },
                PreviousState::Hypothesis(previous),
            ) => {
                let (knowledge, _, _) = memory.parts_mut();

                knowledge.restore_hypothesis(rule, previous);
            }
            (Correction::ObsoleteFact { statement }, PreviousState::Fact(previous)) => {
                let (knowledge, _, _) = memory.parts_mut();

                knowledge.restore_entry(statement, previous);
            }
            (
                Correction::ReassessInvocation { episode, step, .. },
                PreviousState::Outcome(previous),
            ) => {
                memory.corrections_mut().reassess(*episode, *step, previous);
                memory.relearn_routing();
            }
            _ => return Err(format!("correction {id} has an inconsistent record")),
        }

        if let Some(record) = memory.corrections_mut().record_mut(id) {
            record.revert(reason);
        }

        Ok(())
    }
}
