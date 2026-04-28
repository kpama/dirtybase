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
                    let migrator = Migrator::new(Some(context.clone())).await;
                    if let Ok(db_manager) = context.get::<Manager>().await {
                        match action {
                            MigrateAction::Up => {
                                let result = migrator.up(&db_manager).await;
                                db_manager.close().await;
                                result
                            }
                            MigrateAction::Down => {
                                let result = migrator.down(&db_manager).await;
                                db_manager.close().await;
                                result
                            }
                            MigrateAction::Reset => {
                                let result = migrator.reset(&db_manager).await;
                                db_manager.close().await;
                                result
                            }
                            MigrateAction::Refresh => {
                                let result = migrator.refresh(&db_manager).await;
                                db_manager.close().await;
                                result
                            }
                            MigrateAction::Unknown => {
                                db_manager.close().await;
                                eprintln!("unknown action");
                                Err(anyhow!("unknown action"))
                            }
                        }
                    } else {
                        eprintln!("could not get database manager");
                        tracing::error!("could not get database manager");
                        Err(anyhow!("could not get database manager"))
                    }
                }
            }
        })
    });

    // Seeding
    let mut seed = clap::Command::new("seed")
        .about("Seed the database with dummy data")
        .arg_required_else_help(true)
        .arg(Arg::new("name").short('n').action(ArgAction::Set));

    seed = seed.subcommand(clap::Command::new("list").about("List all seeders"));

    manager.register(seed, |_, matches, context| {
        Box::pin(async move {
            let manager = if let Ok(manager) = context.get::<Manager>().await {
                manager
            } else {
                return Ok(());
            };
            match matches.subcommand_name() {
                Some(sub_name) => {
                    if sub_name == "list" {
                        let seeder = SeederRegisterer::new("list-all", manager, context.clone());
                        seeder.list().await;
                    }
                }
                None => {
                    if let Some(name) = matches.get_one::<String>("name") {
                        let seeder = SeederRegisterer::new(name, manager, context.clone());
                        seeder.seed().await;
                    }
                }
            }
            Ok(())
        })
    });

    manager
}
