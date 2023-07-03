pub trait TableEntityTrait {
    fn table_columns() -> &'static [&'static str];

    fn table_name() -> &'static str;

    fn column_aliases() -> Vec<String> {
        Self::table_columns()
            .iter()
            .map(|c| format!("{0}.{1} as '{0}.{1}'", Self::table_name(), c))
            .collect()
    }
}
