use crate::cognition::cognitive_module::CognitiveModule;

/// Set of modules available to the engine. Adding a capability only
/// requires registering a new implementation of `CognitiveModule`.
#[derive(Default)]
pub(crate) struct ModuleRegistry {
    modules: Vec<Box<dyn CognitiveModule>>,
}

impl ModuleRegistry {
    pub(crate) fn register(&mut self, module: impl CognitiveModule + 'static) {
        self.modules.push(Box::new(module));
    }

    pub(crate) fn modules(&self) -> &[Box<dyn CognitiveModule>] {
        &self.modules
    }
}
