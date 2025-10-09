use std::sync::Arc;

pub use ulid::*;

pub type UlidString = String;
pub type ArcUlid = Arc<String>;

pub fn generate_ulid() -> UlidString {
    Ulid::new().to_string().to_lowercase()
}

pub fn generate_arc_ulid() -> ArcUlid {
    generate_ulid().into()
}
