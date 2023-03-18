pub mod column;
pub mod helper;
pub mod index;
pub mod join_builder;
pub mod manager;
pub mod query;
pub mod query_conditions;
pub mod query_join_types;
pub mod query_operators;
pub mod query_values;
pub mod save;
pub mod schema;
pub mod table;
pub mod user_table;
pub mod where_join_operators;

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
