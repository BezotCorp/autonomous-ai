mod acquisition;
mod capabilities;
mod cognition;
mod engine;
mod events;
mod interface;
mod mathematics;
mod memory;
mod reasoning;
mod representation;
mod resources;

fn main() -> std::process::ExitCode {
    interface::execution::execute()
}
