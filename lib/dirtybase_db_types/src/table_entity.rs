use crate::types::{FromColumnAndValue, IntoColumnAndValue};

pub trait TableEntityTrait: FromColumnAndValue + IntoColumnAndValue {
    fn table_columns() -> &'static [&'static str];

    fn table_name() -> &'static str;

    fn prefix_with_tbl(subject: &str) -> String {
        format!("{}.{}", Self::table_name(), subject)
    }

    fn table_column_full_names() -> Vec<String> {
        Self::table_columns()
            .iter()
            .map(|c| format!("{0}.{1}", Self::table_name(), c))
            .collect()
    }

    fn column_aliases<'a>(prefix: Option<&'a str>) -> Vec<String> {
        let pre = prefix.unwrap_or(Self::table_name());
        Self::table_columns()
            .iter()
            .map(|c| format!("{0}.{2} as '{1}.{2}'", Self::table_name(), pre, c))
            .collect()
    }
}
