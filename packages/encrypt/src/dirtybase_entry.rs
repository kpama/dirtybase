mod key_rotation;

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
    async fn register_cli_commands(&self, mut manager: CliCommandManager) -> CliCommandManager {
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
        manager.register(command, |_name, matches, context| {
            Box::pin(async move {
                if let Some(("keygen", arg)) = matches.subcommand() {
                    let key = Encrypter::key_to_env_value(&Encrypter::generate_aes256gcm_key());
                    if arg.get_flag("print") {
                        println!("{key}");
                    } else {
                        let config = context.get::<DirtyConfig>().await?;
                        let keys = key_rotation::ConfiguredKeys::load(&config).await?;
                        let path = key_rotation::write_key(&config, &keys, &key)?;
                        println!("wrote encryption key to {}", path.display());
                    }
                }
                Ok(())
            })
        });
        manager
    }
}
