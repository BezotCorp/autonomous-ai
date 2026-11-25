use crate::cognitive_processor::CognitiveProcessor;

#[derive(Default)]
pub(crate) struct CognitiveRegistry {
    processors: Vec<Box<dyn CognitiveProcessor>>,
}

impl CognitiveRegistry {
    pub(crate) fn register(&mut self, processor: impl CognitiveProcessor + 'static) {
        self.processors.push(Box::new(processor));
    }

    pub(crate) fn processors(&self) -> impl Iterator<Item = &dyn CognitiveProcessor> {
        self.processors.iter().map(Box::as_ref)
    }
}
