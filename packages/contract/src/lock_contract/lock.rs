use crate::lock_contract::LockData;

pub struct Lock {
    data: LockData,
}

impl Lock {
    pub async fn acquire(&self, wait_for: i64) -> bool {
        false
    }

    pub async fn release(self) {
        //
    }
}
