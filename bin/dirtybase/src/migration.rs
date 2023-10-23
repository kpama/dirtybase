use clap::Subcommand;

pub mod migrator;

#[derive(Subcommand, Debug)]
pub enum MigrateAction {
    /// Migrate up
    Up,
    /// Migrate down
    Down,
    /// Resets and migrate all up
    Refresh,
    /// Migrate all down
    Reset,
}
