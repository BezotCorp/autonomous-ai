mod acquisition;
mod capabilities;
mod cognition;
mod consumables;
mod engine;
mod events;
mod individuality;
mod influences;
mod interface;
mod latent;
mod living;
mod mathematics;
mod memory;
mod planning;
mod reasoning;
mod representation;
mod resources;
mod world;

fn main() -> std::process::ExitCode {
    interface::execution::execute()
}
