use ulid::Ulid;

pub type UlidString = String;

pub fn generate_ulid() -> UlidString {
    Ulid::new().to_string().to_lowercase()
}

/// Transform a table name to a column name.
/// The table name is appended with `_id`
pub fn to_fk_column(foreign_table: &str) -> String {
    format!("{}_id", foreign_table.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_that_foreign_keys_ends_with_id() {
        let name = to_fk_column("foo");
        assert_eq!(name, "foo_id");
    }
}
