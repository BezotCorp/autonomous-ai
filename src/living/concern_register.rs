use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    living::{concern::Concern, concern_status::ConcernStatus},
    representation::{bindings::Bindings, statement::Statement},
};

/// The concerns of an existence, active or concluded.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct ConcernRegister {
    concerns: Vec<Concern>,
}

impl ConcernRegister {
    /// Salience below which a concern has faded away.
    pub(crate) const FADED: f64 = 0.05;
    /// Bound on active concerns; the least salient ones are displaced.
    const MAXIMUM_ACTIVE: usize = 128;
    /// Bound on concluded concerns kept for observation.
    const MAXIMUM_CONCLUDED: usize = 2_000;

    pub(crate) fn concerns(&self) -> &[Concern] {
        &self.concerns
    }

    pub(crate) fn active(&self) -> impl Iterator<Item = &Concern> {
        self.concerns.iter().filter(|concern| concern.is_active())
    }

    pub(crate) fn get_mut(&mut self, id: Uuid) -> Option<&mut Concern> {
        self.concerns.iter_mut().find(|concern| concern.id() == id)
    }

    pub(crate) fn get(&self, id: Uuid) -> Option<&Concern> {
        self.concerns.iter().find(|concern| concern.id() == id)
    }

    /// Forms a concern, or reactivates the active one about the same
    /// subject. Returns its identifier and whether it is new, and the
    /// concerns displaced to stay within bounds.
    pub(crate) fn form(
        &mut self,
        subject: Statement,
        origin: String,
        salience: f64,
        lived: f64,
    ) -> (Uuid, bool, Vec<Uuid>) {
        if let Some(existing) = self
            .concerns
            .iter_mut()
            .find(|concern| concern.is_active() && concern.subject() == &subject)
        {
            existing.activate(salience, lived);
            return (existing.id(), false, Vec::new());
        }

        let concern = Concern::new(subject, origin, salience, lived);
        let id = concern.id();

        self.concerns.push(concern);

        let mut displaced = Vec::new();

        while self.active().count() > Self::MAXIMUM_ACTIVE {
            let Some(weakest) = self
                .active()
                .filter(|concern| concern.id() != id)
                .min_by(|left, right| left.salience().total_cmp(&right.salience()))
                .map(Concern::id)
            else {
                break;
            };

            if let Some(concern) = self.get_mut(weakest) {
                concern.conclude(ConcernStatus::Faded);
            }

            displaced.push(weakest);
        }

        self.prune();
        (id, true, displaced)
    }

    /// Fades every active concern by a factor; returns the ones that faded
    /// away.
    pub(crate) fn fade(&mut self, factor: f64) -> Vec<Uuid> {
        let mut faded = Vec::new();

        for concern in self
            .concerns
            .iter_mut()
            .filter(|concern| concern.is_active())
        {
            concern.fade(factor);

            if concern.salience() < Self::FADED {
                concern.conclude(ConcernStatus::Faded);
                faded.push(concern.id());
            }
        }

        faded
    }

    /// Reactivates the active concerns about one of these predicates.
    pub(crate) fn reactivate(&mut self, predicates: &[String], gain: f64, lived: f64) -> Vec<Uuid> {
        let mut reactivated = Vec::new();

        for concern in self
            .concerns
            .iter_mut()
            .filter(|concern| concern.is_active())
        {
            if predicates
                .iter()
                .any(|predicate| predicate == concern.subject().predicate())
            {
                concern.activate(gain, lived);
                reactivated.push(concern.id());
            }
        }

        reactivated
    }

    /// Concludes the active concerns that a statement answers.
    pub(crate) fn resolve(&mut self, answers: &[Statement]) -> Vec<Uuid> {
        let mut resolved = Vec::new();

        for concern in self
            .concerns
            .iter_mut()
            .filter(|concern| concern.is_active())
        {
            if answers.iter().any(|answer| {
                Bindings::default()
                    .unify(concern.subject(), answer)
                    .is_some()
            }) {
                concern.conclude(ConcernStatus::Resolved);
                resolved.push(concern.id());
            }
        }

        resolved
    }

    fn prune(&mut self) {
        let concluded = self
            .concerns
            .iter()
            .filter(|concern| !concern.is_active())
            .count();

        if concluded > Self::MAXIMUM_CONCLUDED {
            let mut excess = concluded - Self::MAXIMUM_CONCLUDED;

            self.concerns.retain(|concern| {
                if excess > 0 && !concern.is_active() {
                    excess -= 1;
                    false
                } else {
                    true
                }
            });
        }
    }
}
