use crate::{
    cognition::{
        cognitive_module::CognitiveModule, module_context::ModuleContext,
        module_descriptor::ModuleDescriptor, module_output::ModuleOutput, proposal::Proposal,
    },
    memory::imagined_capability_spec::ImaginedCapabilitySpec,
    reasoning::epistemic_kind::EpistemicKind,
    representation::{
        bindings::Bindings, content::Content, content_kind::ContentKind, provenance::Provenance,
        statement::Statement, term::Term,
    },
};

/// Simulation, inside a dream, of a capability imagined there. It answers
/// the needs it is imagined to supply with invented statements: the form
/// it was imagined with, completed by the need, the rest invented. Its
/// results are `Imagined`; it is only instantiated in dream episodes.
pub(crate) struct ImaginedCapability {
    descriptor: ModuleDescriptor,
    spec: ImaginedCapabilitySpec,
}

impl ImaginedCapability {
    pub(crate) fn new(spec: ImaginedCapabilitySpec) -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                &format!("imagined:{}", spec.name()),
                "Capability imagined in a dream, simulated there only.",
            )
            .consuming(&[ContentKind::Need])
            .supplying(&[spec.supplies()])
            .producing(&[EpistemicKind::Imagined]),
            spec,
        }
    }

    /// Completes a statement: variables left by the need become invented
    /// entities.
    fn complete(statement: &Statement, counter: &mut usize) -> Statement {
        let arguments = statement
            .arguments()
            .iter()
            .map(|argument| match argument {
                Term::Variable(name) => {
                    *counter += 1;
                    Term::symbol(format!("imagined_{name}_{counter}"))
                }
                other => other.clone(),
            })
            .collect();

        if statement.is_negated() {
            Statement::negative(statement.predicate(), arguments)
        } else {
            Statement::new(statement.predicate(), arguments)
        }
    }
}

impl CognitiveModule for ImaginedCapability {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let mut output = ModuleOutput::default();
        let mut counter = 0;
        let provenance = Provenance::new(
            "imagination",
            Some(format!("capability:{}", self.spec.name())),
            None,
        );

        for (_, need) in context.open_needs(&self.descriptor) {
            for answer in self.spec.answers() {
                let Some(bindings) = Bindings::default().unify(answer, need) else {
                    continue;
                };

                output.propose(
                    Proposal::new(
                        Content::Statement(Self::complete(&bindings.apply(answer), &mut counter)),
                        EpistemicKind::Imagined,
                        Vec::new(),
                    )
                    .with_provenance(provenance.clone()),
                );
            }
        }

        Ok(output)
    }
}
