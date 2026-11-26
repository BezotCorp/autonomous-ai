use crate::{
    cognition::{
        cognitive_module::CognitiveModule, module_context::ModuleContext,
        module_descriptor::ModuleDescriptor, module_output::ModuleOutput, proposal::Proposal,
    },
    reasoning::epistemic_kind::EpistemicKind,
    representation::{
        bindings::Bindings, content::Content, content_kind::ContentKind, statement::Statement,
    },
};

/// Test double of an acquisition capability: when one of the needs it
/// supplies is open, it instantiates its output template for every known
/// instance of its prerequisite, or expresses the prerequisite as a need.
pub(crate) struct ScriptedModule {
    descriptor: ModuleDescriptor,
    prerequisite: Option<Statement>,
    template: Statement,
}

impl ScriptedModule {
    pub(crate) fn new(name: &str, prerequisite: Option<&str>, template: &str) -> Self {
        let template: Statement = template.parse().unwrap();
        let signature = template.signature();

        Self {
            descriptor: ModuleDescriptor::new(name, "test double")
                .consuming(&[ContentKind::Need])
                .supplying(&[signature.as_str()])
                .producing(&[EpistemicKind::Observation, EpistemicKind::Need]),
            prerequisite: prerequisite.map(|text| text.parse().unwrap()),
            template,
        }
    }
}

impl CognitiveModule for ScriptedModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String> {
        let mut output = ModuleOutput::default();
        let needs: Vec<_> = context
            .open_needs(&self.descriptor)
            .into_iter()
            .map(|(id, _)| id)
            .collect();

        if needs.is_empty() {
            return Ok(output);
        }

        let Some(prerequisite) = &self.prerequisite else {
            output.propose(Proposal::new(
                Content::Statement(self.template.clone()),
                EpistemicKind::Observation,
                Vec::new(),
            ));
            return Ok(output);
        };

        let matches: Vec<Bindings> = context
            .graph()
            .matching(prerequisite, &Bindings::default())
            .map(|(_, bindings)| bindings)
            .collect();

        if matches.is_empty() {
            output.propose(Proposal::new(
                Content::Need(prerequisite.clone()),
                EpistemicKind::Need,
                needs,
            ));
        }

        for bindings in matches {
            output.propose(Proposal::new(
                Content::Statement(bindings.apply(&self.template)),
                EpistemicKind::Observation,
                Vec::new(),
            ));
        }

        Ok(output)
    }
}
