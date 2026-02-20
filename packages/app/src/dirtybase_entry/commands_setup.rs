use dirtybase_contract::cli_contract::CliCommandManager;

use crate::run_http;

pub(crate) fn register(mut manager: CliCommandManager) -> CliCommandManager {
    // serve command
    let serve = clap::Command::new("serve").about("Start the web application server");

    manager.register(serve, |_, _, context| {
        Box::pin(async move {
            let ci = context.container();
            run_http(ci.get_type().await.expect("could not get app service")).await
        })
    });

    manager
}
