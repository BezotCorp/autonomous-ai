mod arguments;
mod capability;
mod cargo_manifest;
mod cargo_observer;
mod constants;
mod dependency;
mod event;
mod event_payload;
mod execution;
mod execution_report;
mod github;
mod intelligence;
mod memory;
mod memory_store;
mod observation;
mod observation_failure;
mod observation_payload;
mod repository;
mod repository_snapshot;

fn main() -> std::process::ExitCode {
    execution::execute()
}
