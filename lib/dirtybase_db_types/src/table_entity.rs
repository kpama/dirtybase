use crate::types::{FromColumnAndValue, IntoColumnAndValue};

pub trait TableEntityTrait: FromColumnAndValue + IntoColumnAndValue {
    fn table_columns() -> &'static [&'static str];

    fn table_name() -> &'static str;

    fn id_column() -> Option<&'static str> {
        None
    }

    fn foreign_id_column() -> Option<&'static str> {
        None
    }

    fn prefix_with_tbl<T: ToString>(subject: T) -> String {
        format!("{}.{}", Self::table_name(), subject.to_string())
    }

    fn table_column_full_names() -> Vec<String> {
        Self::table_columns()
            .iter()
            .map(|c| format!("{0}.{1}", Self::table_name(), c))
            .collect()
    }

    fn column_aliases<T: ToString>(prefix: Option<T>) -> Vec<String> {
        let pre: String = if let Some(t) = prefix {
            t.to_string()
        } else {
            Self::table_name().to_owned()
        };

        Self::table_columns()
            .iter()
            .map(|c| format!("{0}.{2} as '{1}.{2}'", Self::table_name(), pre, c))
            .collect()
    }
}
