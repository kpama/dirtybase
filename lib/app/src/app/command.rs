use clap::{ArgMatches, Parser, Subcommand};

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

impl From<(String, ArgMatches)> for Commands {
    fn from(value: (String, ArgMatches)) -> Self {
        match value {
            (name, _) if name == "serve" => Commands::Serve,
            (name, mut args) if name == "migrate" && args.subcommand().is_some() => {
                Commands::Migrate {
                    action: migrator::MigrateAction::from(args.remove_subcommand().unwrap()),
                }
            }
            (name, mut args) if name == "queue" => Commands::Queue {
                name: args.remove_one("cluster"),
            },
            (name, mut args) if name == "handle" => Commands::Handle {
                cluster: args.remove_one("cluster"),
            },
            v @ _ => panic!("{} is not a valid command", &v.0),
        }
    }
}
