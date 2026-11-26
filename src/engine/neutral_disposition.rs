use crate::{
    engine::{disposition::Disposition, modulation::Modulation},
    latent::occurrence::Occurrence,
};

/// Reasoning without any internal state: the laboratory condition, and the
/// condition of every episode of earlier generations.
pub(crate) struct NeutralDisposition;

impl Disposition for NeutralDisposition {
    fn modulation(&self) -> Modulation {
        Modulation::neutral()
    }

    fn feel(&mut self, _occurrence: Occurrence) {}
}
