use std::error::Error;

use crate::{
    capability::Capability, cargo_observer::CargoObserver, event::Event,
    event_payload::EventPayload, execution_report::ExecutionReport, github::Github, memory::Memory,
    memory_store::MemoryStore, observation::Observation, observation_failure::ObservationFailure,
    observation_payload::ObservationPayload, repository::Repository,
};

pub(crate) struct Intelligence {
    memory: Memory,
}

impl Intelligence {
    pub(crate) fn new(memory: Memory) -> Self {
        Self { memory }
    }

    pub(crate) fn receive(
        &mut self,
        event: Event,
        store: &MemoryStore,
    ) -> Result<ExecutionReport, Box<dyn Error>> {
        let event_id = event.id().to_owned();

        if self.memory.contains_event(&event_id) {
            return Ok(ExecutionReport::new(
                event_id,
                true,
                0,
                0,
                self.memory.event_count(),
                self.memory.observation_count(),
            ));
        }

        let previous_observations = self.memory.observation_count();

        let previous_failures = self.memory.failure_count();

        match event.payload() {
            EventPayload::RepositoryChanged {
                repository,
                revision,
            } => {
                self.observe_repository(&event_id, repository, revision);
            }
            EventPayload::PackagePublished { repository, .. } => {
                match Github::repositories(repository.owner()) {
                    Ok(repositories) => {
                        self.record(
                            &event_id,
                            ObservationPayload::RepositoriesDiscovered {
                                repositories: repositories.clone(),
                            },
                        );

                        for candidate in repositories {
                            self.observe_repository(&event_id, &candidate, "HEAD");
                        }
                    }
                    Err(error) => {
                        self.record(
                            &event_id,
                            ObservationPayload::Failed {
                                failure: ObservationFailure::new(
                                    Capability::DiscoverRepositories,
                                    repository.clone(),
                                    error.to_string(),
                                ),
                            },
                        );
                    }
                }
            }
            EventPayload::WorkflowCompleted {
                repository,
                revision,
                ..
            } => {
                self.observe_repository(&event_id, repository, revision);
            }
        }

        self.memory.record_event(event);

        store.save(&self.memory)?;

        Ok(ExecutionReport::new(
            event_id,
            false,
            self.memory.observation_count() - previous_observations,
            self.memory.failure_count() - previous_failures,
            self.memory.event_count(),
            self.memory.observation_count(),
        ))
    }

    fn observe_repository(&mut self, event_id: &str, repository: &Repository, reference: &str) {
        match CargoObserver::observe(repository, reference) {
            Ok(snapshot) => {
                self.record(
                    event_id,
                    ObservationPayload::RepositoryObserved { snapshot },
                );
            }
            Err(error) => {
                self.record(
                    event_id,
                    ObservationPayload::Failed {
                        failure: ObservationFailure::new(
                            Capability::ObserveRepository,
                            repository.clone(),
                            error.to_string(),
                        ),
                    },
                );
            }
        }
    }

    fn record(&mut self, event_id: &str, payload: ObservationPayload) {
        self.memory
            .record_observation(Observation::new(event_id.to_owned(), payload));
    }
}
