use crate::db_contract::base::table::{CREATED_AT_FIELD, DELETED_AT_FIELD, UPDATED_AT_FIELD};

use super::{
    query_column::QueryColumn,
    types::{FromColumnAndValue, ToColumnAndValue},
};

pub trait TableModel: FromColumnAndValue + ToColumnAndValue {
    /// Tables table's column names without prefix
    fn table_columns() -> &'static [&'static str];

    /// Table name
    fn table_name() -> &'static str;

    fn id_field() -> &'static str {
        "id"
    }

    /// Table's ID column. Usually `id`
    fn id_column() -> &'static str {
        "id"
    }

    /// Table's foreign column name `table name + _ + id`
    fn foreign_id_column() -> &'static str;

    fn entity_hash(&self) -> u64;

    fn created_at_column() -> Option<&'static str> {
        Some(CREATED_AT_FIELD)
    }

    fn updated_at_column() -> Option<&'static str> {
        Some(UPDATED_AT_FIELD)
    }

    fn deleted_at_column() -> Option<&'static str> {
        Some(DELETED_AT_FIELD)
    }

    fn prefix_with_tbl<T: ToString>(subject: T) -> String {
        format!("{}.{}", Self::table_name(), subject.to_string())
    }

    /// Returns the full column names as strings
    fn table_column_full_names() -> Vec<String> {
        Self::table_columns()
            .iter()
            .map(|c| format!("{0}.{1}", Self::table_name(), c))
            .collect()
    }

    /// Returns the full columns names as QueryColumn
    fn table_query_columns() -> Vec<QueryColumn> {
        Self::table_columns()
            .iter()
            .map(|c| QueryColumn::new(*c, Some(Self::table_name()), None))
            .collect()
    }

    fn table_query_col_aliases(prefix: Option<&str>) -> Vec<QueryColumn> {
        let pre: String = if let Some(t) = prefix {
            t.to_string()
        } else {
            Self::table_name().to_owned()
        };

        Self::table_columns()
            .iter()
            .map(|c| {
                let alias = format!("{}.{}", pre, c);
                QueryColumn::new(*c, Some(Self::table_name()), Some(&alias))
            })
            .collect()
    }

    fn column_aliases(prefix: Option<&str>) -> Vec<String> {
        let pre: String = if let Some(t) = prefix {
            t.to_string()
        } else {
            Self::table_name().to_owned()
        };

        Self::table_columns()
            .iter()
            .map(|c| format!("{0}.{2} as \"{1}.{2}\"", Self::table_name(), pre, c))
            .collect()
    }
}
