use std::error::Error;

use crate::{
    capability::Capability, cargo_acquisition::CargoAcquisition, cargo_observer::CargoObserver,
    event::Event, event_payload::EventPayload, execution_report::ExecutionReport, github::Github,
    memory::Memory, observation_failure::ObservationFailure,
    observation_payload::ObservationPayload, publication_analyzer::PublicationAnalyzer,
    repository::Repository, snapshot_identity::SnapshotIdentity,
};

pub(crate) struct Intelligence {
    memory: Memory,
}

impl Intelligence {
    pub(crate) fn new(memory: Memory) -> Self {
        Self { memory }
    }

    pub(crate) fn memory(&self) -> &Memory {
        &self.memory
    }

    pub(crate) fn receive(&mut self, event: Event) -> Result<ExecutionReport, Box<dyn Error>> {
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
            }
            | EventPayload::WorkflowCompleted {
                repository,
                revision,
                ..
            } => {
                self.observe_repository(&event_id, repository, revision);
            }

            EventPayload::PackagePublished {
                repository,
                package,
                registry,
                version,
            } => {
                self.analyze_publication(&event_id, repository, package, registry, version)?;
            }
        }

        self.memory.record_event(event);

        Ok(ExecutionReport::new(
            event_id,
            false,
            self.memory.observation_count() - previous_observations,
            self.memory.failure_count() - previous_failures,
            self.memory.event_count(),
            self.memory.observation_count(),
        ))
    }

    fn analyze_publication(
        &mut self,
        event_id: &str,
        publisher: &Repository,
        package: &str,
        registry: &str,
        version: &semver::Version,
    ) -> Result<(), Box<dyn Error>> {
        let repositories = match Github::repositories(publisher.owner()) {
            Ok(repositories) => repositories,

            Err(error) => {
                self.record_failure(event_id, Capability::DiscoverRepositories, publisher, error);

                return Ok(());
            }
        };

        self.memory.record_observation(
            event_id,
            ObservationPayload::RepositoriesDiscovered {
                repositories: repositories.clone(),
            },
        );

        let mut identities = Vec::new();

        let acquisitions =
            CargoObserver::acquire_many(self.memory.knowledge(), &repositories, "HEAD");

        for (repository, result) in acquisitions {
            if let Some(identity) = self.integrate_acquisition(event_id, &repository, result) {
                identities.push(identity);
            }
        }

        let analysis = PublicationAnalyzer::analyze(
            event_id,
            self.memory.knowledge(),
            identities,
            package,
            registry,
            version,
        )?;

        self.memory.record_publication_analysis(analysis);

        Ok(())
    }

    fn observe_repository(
        &mut self,
        event_id: &str,
        repository: &Repository,
        reference: &str,
    ) -> Option<SnapshotIdentity> {
        let result = CargoObserver::acquire(self.memory.knowledge(), repository, reference)
            .map_err(|error| error.to_string());

        self.integrate_acquisition(event_id, repository, result)
    }

    fn integrate_acquisition(
        &mut self,
        event_id: &str,
        repository: &Repository,
        result: Result<CargoAcquisition, String>,
    ) -> Option<SnapshotIdentity> {
        match result {
            Ok(acquisition) => {
                let (identity, reused) = acquisition.integrate(&mut self.memory);
                self.memory.record_observation(
                    event_id,
                    ObservationPayload::RepositoryObserved {
                        identity: identity.clone(),
                        reused,
                    },
                );

                Some(identity)
            }

            Err(error) => {
                self.record_failure(
                    event_id,
                    Capability::ObserveRepository,
                    repository,
                    error.into(),
                );

                None
            }
        }
    }

    fn record_failure(
        &mut self,
        event_id: &str,
        capability: Capability,
        repository: &Repository,
        error: Box<dyn Error>,
    ) {
        self.memory.record_observation(
            event_id,
            ObservationPayload::Failed {
                failure: ObservationFailure::new(capability, repository.clone(), error.to_string()),
            },
        );
    }
}
