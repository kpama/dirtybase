use std::sync::Arc;

use ulid::Ulid;

pub type UlidString = String;
pub type ArcUlid = Arc<str>;

pub fn generate_ulid() -> UlidString {
    Ulid::new().to_string().to_lowercase()
}

pub fn generate_arc_ulid() -> ArcUlid {
    generate_ulid().into()
}

/// Transform a table name to a column name.
/// The table name is appended by the table's primary key or defaults to `_id`
pub fn to_fk_column(foreign_table: &str, id: Option<&str>) -> String {
    format!(
        "{}_{}",
        foreign_table.to_ascii_lowercase(),
        id.unwrap_or("id")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_that_foreign_keys_ends_with_id() {
        let name = to_fk_column("foo", None);
        assert_eq!(name, "foo_id");
    }

    #[test]
    fn test_that_foreign_keys_ends_with_key() {
        let name = to_fk_column("foo", Some("key"));
        assert_eq!(name, "foo_key");
    }
}
