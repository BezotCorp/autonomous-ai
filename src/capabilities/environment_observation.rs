use crate::{
    cognition::{
        cognitive_module::CognitiveModule, module_context::ModuleContext,
        module_descriptor::ModuleDescriptor, module_output::ModuleOutput, proposal::Proposal,
    },
    reasoning::epistemic_kind::EpistemicKind,
    representation::{
        bindings::Bindings, content::Content, content_kind::ContentKind, provenance::Provenance,
        statement::Statement, term::Term,
    },
    world::portal::Portal,
};

/// Looks around: answers `present(?x)` with the presences of the
/// environment that happen to be perceptible now, each under the opaque
/// address the environment knows it by. It is used, like any capability,
/// only if a need calls for it; nothing tells the individual who is there.
/// In imagination it perceives nothing: an imagined presence is never a
/// perceived one.
pub(crate) struct EnvironmentObservation {
    descriptor: ModuleDescriptor,
    portal: Portal,
}

impl EnvironmentObservation {
    pub(crate) fn new(portal: Portal) -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "environment_observation",
                "Perceives the presences of the environment that are perceptible now.",
            )
            .consuming(&[ContentKind::Need])
            .supplying(&["present/1"])
            .producing(&[EpistemicKind::Observation]),
            portal,
        }
    }
}

impl CognitiveModule for EnvironmentObservation {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let mut output = ModuleOutput::default();
        let needs = context.open_needs(&self.descriptor);

        if needs.is_empty() {
            return Ok(output);
        }

        if !context.access().is_live() {
            output.note("nothing is perceived of the environment in imagination");
            return Ok(output);
        }

        let perceived = self.portal.perceive();
        let provenance = Provenance::new(
            "environment",
            None,
            Some(format!("{:.0}", self.portal.now())),
        );

        if perceived.is_empty() {
            output.note("no presence perceived");
        }

        for handle in perceived {
            let statement = Statement::new("present", vec![Term::symbol(handle)]);

            if needs
                .iter()
                .any(|(_, pattern)| Bindings::default().unify(pattern, &statement).is_some())
            {
                output.propose(
                    Proposal::new(
                        Content::Statement(statement),
                        EpistemicKind::Observation,
                        Vec::new(),
                    )
                    .with_provenance(provenance.clone()),
                );
            }
        }

        Ok(output)
    }
}
