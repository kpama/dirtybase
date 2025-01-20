mod cli_middleware_manager;
mod command_manager;

pub use clap;
pub use cli_middleware_manager::*;
pub use command_manager::*;

pub mod prelude {
    pub use super::cli_middleware_manager::*;
    pub use super::command_manager::*;
    pub use clap::ArgMatches;
    pub use clap::Command;
}
