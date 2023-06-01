use clap::Subcommand;

mod migrator;

#[derive(Subcommand, Debug)]
pub enum MigrateDirection {
    Up,
    Down,
    New { name: String },
}
