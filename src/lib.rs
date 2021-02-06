mod events;
pub mod server;
mod state;
pub mod ui;

use rand::Rng;

pub const DEFAULT_PORT: u16 = 42069;

pub fn generate_id() -> u32 {
    let mut rng = rand::thread_rng();
    rng.gen()
}
