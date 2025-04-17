use dirtybase_contract::prelude::Context;

use crate::JobId;

pub(crate) async fn execute(
    context: Context,
    id: Result<JobId, anyhow::Error>,
) -> Result<(), anyhow::Error> {
    let id = id?;
    println!("handled command to run job: {}", &id);

    Ok(())
}
