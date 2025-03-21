use anyhow::anyhow;
use dirtybase_contract::{
    cli::clap::{self, Arg, ArgAction},
    prelude::*,
};

#[derive(Debug, Default)]
pub struct Extension;

#[async_trait]
impl ExtensionSetup for Extension {
    fn register_cli_commands(&self, mut manager: CliCommandManager) -> CliCommandManager {
        let command = clap::Command::new("encrypt")
            .about("Execute encryption command")
            .arg_required_else_help(true)
            .subcommand(
                clap::Command::new("keygen")
                    .arg(
                        Arg::new("print")
                            .long("print")
                            .short('p')
                            .action(ArgAction::SetTrue)
                            .help("print the key to the console"),
                    )
                    .about("generate encryption key"),
            );
        manager.register(command, |name, matches, context| {
            Box::pin(async move {
                if name.to_lowercase() == "encrypt" {
                    if let Some((name, arg)) = matches.subcommand() {
                        if name == "keygen" {
                            if arg.get_flag("print") {
                                println!(">>>>>>>>>>>>>> generate and printing encryption key");
                            } else {
                                println!(">>>>>>>>>>>>>> generate encryption key");
                            }
                        }
                    }
                    return Ok(());
                }

                Err(anyhow!("unknown command"))
            })
        });
        manager
    }
}
