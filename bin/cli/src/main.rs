use std::process::Command;

use clap::{command, Parser, Subcommand};

mod make_migration;

fn main() {
    let args = Cli::parse();

    let dist_path = if let Some(package) = &args.package {
        read_metadata(package)
    } else {
        read_metadata("")
    };

    match &args.command {
        Commands::Make { what } => match what {
            MakeSubcommand::Migration { name } => {
                let ts = chrono::Utc::now().timestamp();

                let joinned = name.split_whitespace().collect::<String>();
                let filename = joinned.to_lowercase();

                let full_filename = format!("mig_{}_{}", ts, filename);
                let struct_name = format!("Mig{}{}", ts, filename.split('_').collect::<String>());

                let path = dist_path
                    .parent()
                    .unwrap()
                    .join("migration")
                    .join(&full_filename);

                println!("making migration : {:?}", name);
                println!("migration filename : {:?}", full_filename);
                println!("migration struct name: {:?}", struct_name);
                println!("migration full path: {:?}", path);

                make_migration::make(&struct_name, &dist_path, &full_filename);
            }
            _ => (),
        },
        Commands::New { name } => {
            eprintln!("creating a new application")
        }
        Commands::Init => {
            eprintln!("initialising features")
        }
    }
}

fn read_metadata(package: &str) -> std::path::PathBuf {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--no-deps")
        .arg("--format-version=1")
        .output();

    let out = output.unwrap().stdout;
    let o = std::str::from_utf8(&out).unwrap();
    let value: serde_json::Value = serde_json::from_str(o).unwrap();

    let packages = value.get("packages").unwrap().as_array().unwrap();

    let mut path = packages[0].get("targets").unwrap().as_array().unwrap()[0]
        .get("src_path")
        .unwrap();

    if !package.is_empty() {
        let pass_name = package.to_lowercase();
        for pkg in packages {
            match pkg.get("name") {
                Some(value) => {
                    if pass_name == value.as_str().unwrap() {
                        path = pkg.get("targets").unwrap().as_array().unwrap()[0]
                            .get("src_path")
                            .unwrap();
                        break;
                    }
                }
                None => (),
            }
        }
    }

    std::path::PathBuf::from(serde_json::from_value::<String>(path.clone()).unwrap())
        .parent()
        .unwrap()
        .to_path_buf()
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
    /// Initialise DirtyBase feature in the curren directory
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
