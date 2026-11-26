use crate::{engine::modulation::Modulation, latent::occurrence::Occurrence};

/// What the reasoning engine knows of the internal state of whoever
/// reasons: it reads a modulation before every decision and reports every
/// occurrence (an invocation, an error, a consumption, an answer...) as it
/// happens. The modulation may therefore change during an episode, as the
/// occurrences of the episode move the internal state.
pub(crate) trait Disposition {
    fn modulation(&self) -> Modulation;

    fn feel(&mut self, occurrence: Occurrence);
}
