use crate::cognition::{
    module_context::ModuleContext, module_descriptor::ModuleDescriptor, module_output::ModuleOutput,
};

/// Common contract of every cognitive capability. A module transforms the
/// current state of the reasoning into proposals; it never decides by
/// itself when it is mobilised.
pub(crate) trait CognitiveModule {
    fn descriptor(&self) -> &ModuleDescriptor;

    fn apply(&self, context: &ModuleContext<'_>) -> Result<ModuleOutput, String>;
}
