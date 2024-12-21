pub struct MemoryConnector;

impl dirtybase_contract::queue::Connector for MemoryConnector {
    fn fetch(&self) -> i32 {
        0
    }

    fn put(&self, job: i32) {
        log::debug!("queuing job: {}", job);
    }

    fn delete(&self, job: i32) {
        log::debug!("deleting job: {}", job);
    }
}
