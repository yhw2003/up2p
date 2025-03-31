pub mod core;
pub mod utils;
pub mod client_lib;
// pub mod state;

pub fn get_binencode_config() -> bincode::config::Configuration {
    bincode::config::standard()
}