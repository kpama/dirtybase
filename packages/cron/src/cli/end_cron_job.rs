use dirtybase_contract::prelude::Context;

use crate::JobId;

pub(crate) async fn execute(
    _context: Context,
    id: Result<JobId, anyhow::Error>,
) -> Result<(), anyhow::Error> {
    let id = id?;
    println!("ending job {} and removing it from the scheduler", &id);
    Ok(())
}
