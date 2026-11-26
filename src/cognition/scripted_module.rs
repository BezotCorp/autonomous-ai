use crate::{
    acquisition::access_mode::AccessMode,
    cognition::{
        cognitive_module::CognitiveModule, module_context::ModuleContext,
        module_descriptor::ModuleDescriptor, module_output::ModuleOutput, proposal::Proposal,
    },
    memory::cached_response::CachedResponse,
    reasoning::epistemic_kind::EpistemicKind,
    representation::{
        bindings::Bindings, content::Content, content_kind::ContentKind, provenance::Provenance,
        statement::Statement,
    },
};

/// Test double of an acquisition capability: when one of the needs it
/// supplies is open, it instantiates its output template for every known
/// instance of its prerequisite, or expresses the prerequisite as a need.
/// It may describe an observed scope (source, scope, revision) and consume
/// units of an external resource at each acquisition.
pub(crate) struct ScriptedModule {
    descriptor: ModuleDescriptor,
    prerequisite: Option<Statement>,
    template: Statement,
    provenance: Option<(String, String, String)>,
    consumption: Option<(String, f64)>,
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
            provenance: None,
            consumption: None,
        }
    }

    pub(crate) fn observing(mut self, source: &str, scope: &str, revision: &str) -> Self {
        self.provenance = Some((source.to_owned(), scope.to_owned(), revision.to_owned()));
        self
    }

    pub(crate) fn consuming(mut self, resource: &str, units: f64) -> Self {
        self.consumption = Some((resource.to_owned(), units));
        self
    }

    fn record_key(&self) -> String {
        format!("scripted:{}", self.descriptor.name())
    }

    /// Records what it acquired, as a real acquisition module would.
    fn record(&self, context: &ModuleContext<'_>, statements: &[Statement]) {
        let body = statements
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n");

        context
            .access()
            .store()
            .put(&self.record_key(), CachedResponse::mutable(body, None));
    }

    fn observation(&self, statement: Statement) -> Proposal {
        let proposal = Proposal::new(
            Content::Statement(statement),
            EpistemicKind::Observation,
            Vec::new(),
        );

        match &self.provenance {
            Some((source, scope, revision)) => proposal.with_provenance(Provenance::new(
                source.clone(),
                Some(scope.clone()),
                Some(revision.clone()),
            )),
            None => proposal,
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

        // In a replay, it can only reproduce what it recorded while live.
        if let AccessMode::Replay { as_of, .. } = context.access().mode() {
            let recorded = context
                .access()
                .store()
                .recorded(&self.record_key(), as_of)
                .ok_or_else(|| format!("{}: not recorded, left missing", self.descriptor.name()))?;

            for line in recorded.body().lines() {
                output.propose(self.observation(line.parse()?));
            }

            return Ok(output);
        }

        if let Some((resource, units)) = &self.consumption {
            let meter = context.access().meter();

            meter.authorize(resource, *units)?;
            meter.record(resource, self.descriptor.name(), *units);
        }

        let Some(prerequisite) = &self.prerequisite else {
            self.record(context, std::slice::from_ref(&self.template));
            output.propose(self.observation(self.template.clone()));
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

        let observed: Vec<Statement> = matches
            .iter()
            .map(|bindings| bindings.apply(&self.template))
            .collect();

        if !observed.is_empty() {
            self.record(context, &observed);
        }

        for statement in observed {
            output.propose(self.observation(statement));
        }

        Ok(output)
    }
}
