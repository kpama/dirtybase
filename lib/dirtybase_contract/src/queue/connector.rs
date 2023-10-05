pub trait Connector {
    fn fetch(&self) -> i32; // for now..
    fn put(&self, job: i32);
    fn delete(&self, job: i32);
}
