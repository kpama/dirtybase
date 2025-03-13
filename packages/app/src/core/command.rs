use clap::{ArgMatches, Parser, Subcommand};

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
    Unknown,
}

impl From<(String, ArgMatches)> for Commands {
    fn from(value: (String, ArgMatches)) -> Self {
        match value {
            (name, _) if name == "serve" => Commands::Serve,
            _ => Commands::Unknown,
        }
    }
}
