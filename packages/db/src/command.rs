mod migrator;
use anyhow::anyhow;
use dirtybase_contract::{
    cli_contract::{
        CliCommandManager,
        clap::{self, Arg, ArgAction, ArgMatches},
    },
    db_contract::{SeederRegisterer, base::manager::Manager},
};
use migrator::{MigrateAction, Migrator};

#[derive(Debug, Clone)]
pub(crate) enum Commands {
    Migrate { action: MigrateAction },
}

impl From<(String, ArgMatches)> for Commands {
    fn from(value: (String, ArgMatches)) -> Self {
        match value {
            (name, mut args) if name.to_lowercase() == "migrate" && args.subcommand().is_some() => {
                Commands::Migrate {
                    action: MigrateAction::from(args.remove_subcommand().unwrap()),
                }
            }
            v => panic!("{} is not a valid command", &v.0),
        }
    }
}

pub(crate) fn setup_commands(mut manager: CliCommandManager) -> CliCommandManager {
    // migrate command
    let migrate = clap::Command::new("migrate")
        .about("Execute migration")
        .arg_required_else_help(true)
        .subcommand(clap::Command::new("up").about("Migrate up"))
        .subcommand(clap::Command::new("down").about("Migrate down"))
        .subcommand(clap::Command::new("refresh").about("Resets and migrate all up"))
        .subcommand(clap::Command::new("reset").about("Migrate all down"));

    // -
    manager.register(migrate, |name, matches, context| {
        Box::pin(async move {
            let command: Commands = Commands::from((name, matches));
            match command {
                Commands::Migrate { action } => {
                    let migrator = Migrator::new().await;
                    if let Ok(db_manager) = context.get::<Manager>().await {
                        match action {
                            MigrateAction::Up => return migrator.up(&db_manager).await,
                            MigrateAction::Down => return migrator.down(&db_manager).await,
                            MigrateAction::Reset => return migrator.reset(&db_manager).await,
                            MigrateAction::Refresh => return migrator.refresh(&db_manager).await,
                            MigrateAction::Unknown => {
                                eprintln!("unknown action");
                                return Err(anyhow!("unknown action"));
                            }
                        }
                    } else {
                        eprintln!("could not get database manager");
                        tracing::error!("could not get database manager");
                        return Err(anyhow!("could not get database manager"));
                    }
                }
            }
        })
    });

    // Seeding
    let seed = clap::Command::new("seed")
        .about("Seed the database with dummy data")
        .arg_required_else_help(true)
        .arg(Arg::new("name").short('n').action(ArgAction::Set));

    manager.register(seed, |_, matches, context| {
        Box::pin(async move {
            if let Some(name) = matches.get_one::<String>("name") {
                if let Ok(manager) = context.get::<Manager>().await {
                    let seeder = SeederRegisterer::new(name, manager);
                    seeder.seed().await;
                }
            }
            Ok(())
        })
    });

    manager
}
