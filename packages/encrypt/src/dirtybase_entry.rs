use dirtybase_contract::{
    cli_contract::{
        CliCommandManager,
        clap::{self, Arg, ArgAction},
    },
    prelude::*,
};

use crate::Encrypter;

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
        manager.register(command, |_name, matches, _context| {
            Box::pin(async move {
                if let Some((name, arg)) = matches.subcommand() {
                    if name == "keygen" {
                        // generate the random bytes
                        // base64 encode it
                        let key = format!("base64:{}", Encrypter::generate_aes256gcm_key_string());
                        if arg.get_flag("print") {
                            println!("{}", key);
                        } else {
                            // TODO: WRITE THIS KEY TO THE .ENV FILE
                            //       If there is an existing key, move it to the list of previous keys
                            println!("{}", key);
                        }
                    }
                }
                Ok(())
            })
        });
        manager
    }
}
