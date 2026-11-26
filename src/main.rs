mod acquisition;
mod capabilities;
mod cognition;
mod consumables;
mod engine;
mod events;
mod influences;
mod interface;
mod mathematics;
mod memory;
mod planning;
mod reasoning;
mod representation;
mod resources;

fn main() -> std::process::ExitCode {
    interface::execution::execute()
}
