mod arguments;
mod capability;
mod cargo_acquisition;
mod cargo_lockfile;
mod cargo_manifest;
mod cargo_observer;
mod cargo_snapshot;
mod constants;
mod dependency_declaration;
mod dependency_kind;
mod event;
mod event_payload;
mod execution;
mod execution_report;
mod github;
mod intelligence;
mod knowledge;
mod memory;
mod memory_store;
mod observation;
mod observation_failure;
mod observation_payload;
mod older_resolved_package;
mod publication_analysis;
mod publication_analyzer;
mod registry;
mod repository;
mod resolved_package;
mod snapshot_identity;
mod version_declaration;

fn main() -> std::process::ExitCode {
    execution::execute()
}
