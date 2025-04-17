mod end_cron_job;
mod exit_cron;
mod run_cron_job;
mod start_cron;

use anyhow::anyhow;
use dirtybase_contract::cli_contract::{
    CliCommandManager,
    clap::{self, Arg},
};

use crate::JobId;

pub(crate) fn setup_cli(mut manager: CliCommandManager) -> CliCommandManager {
    let command = clap::Command::new("cron")
        .about("Manage CRON jobs")
        .arg_required_else_help(true)
        .subcommand(clap::Command::new("start").about("Schedules and starts jobs"))
        // $ cron run
        .subcommand(
            clap::Command::new("run")
                .about("Run the job with the specified ID")
                .arg_required_else_help(true)
                .arg(Arg::new("id").help("The ID of the job to run")),
        )
        // $ cron stop foo::bar
        .subcommand(
            clap::Command::new("stop")
                .about("Stops the job with the specified ID")
                .long_about("The specified job will be stopped but not remove from the scheduler")
                .arg_required_else_help(true)
                .arg(Arg::new("id").help("The ID of the job to stop")),
        )
        // $ cron end foo::bar
        .subcommand(
            clap::Command::new("end")
                .about("Ends the job with the specified ID")
                .long_about("The specified job will be stopped but and remove from the scheduler")
                .arg_required_else_help(true)
                .arg(Arg::new("id").help("The ID of the job to end")),
        )
        // $ cron exit
        .subcommand(clap::Command::new("exit").about("Stops all running jobs and exits"));

    manager.register(command, |_name, matches, context| {
        Box::pin(async move {
            if let Some((sub_cmd, args)) = matches.subcommand() {
                let job_id_result = JobId::try_from(args.try_get_one::<String>("id"));

                match sub_cmd {
                    "start" => {
                        return start_cron::execute(context).await;
                    }
                    "run" => {
                        return run_cron_job::execute(context, job_id_result).await;
                    }
                    "end" => {
                        return end_cron_job::execute(context, job_id_result).await;
                    }
                    "exit" => {
                        return exit_cron::execute(context).await;
                    }
                    _ => return Err(anyhow!("Unknown subcommand.")),
                }
            }

            Err(anyhow!("Subcommand not specified."))
        })
    });

    manager
}
