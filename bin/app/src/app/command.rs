use clap::{Parser, Subcommand};

pub mod migrator;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Start the application web server
    Serve,
    /// Execute migration
    Migrate {
        #[command(subcommand)]
        action: migrator::MigrateAction,
    },
    /// Process queued jobs
    Queue { name: Option<String> },
    /// Handle dispatched events
    Handle { cluster: Option<String> },
}
