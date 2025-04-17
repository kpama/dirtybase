use dirtybase_contract::prelude::Context;

pub(crate) async fn execute(context: Context) -> Result<(), anyhow::Error> {
    println!("Ending all jobs and exiting...");
    Ok(())
}
