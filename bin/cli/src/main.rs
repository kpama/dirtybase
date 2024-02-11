use clap::{command, Parser, Subcommand};

mod commands;
mod metadata;

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::New { name } => {
            println!("----> creating a new application: {:?}", name);
            commands::new::create(name);
        }
        Commands::Init => {
            commands::init::init(args.package.as_ref());
        }
        Commands::Make { what } => match what {
            MakeSubcommand::Migration { name } => {
                commands::make_migration::make(args.package.as_ref(), name);
            }
            _ => (),
        },
    }
}

#[derive(Debug, Parser)] // requires `derive` feature
#[command(about = "Dirtybase CLI tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Select the package to run the command against
    #[arg(long, short)]
    package: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialise DirtyBase feature in the current directory
    Init,
    /// Create new application
    New { name: String },
    /// Make a Controller, Model, Event, Handler...
    Make {
        #[command(subcommand)]
        what: MakeSubcommand,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum MakeSubcommand {
    /// Database migration
    Migration {
        /// Migration name
        name: String,
    },
    /// HTTP request handler
    Controller,
    /// Event
    Event,
    /// Event handler
    Handler,
    /// Database table entity
    Entity,
}
