pub trait TableEntityTrait {
    fn columns() -> &'static [&'static str];

    fn table_name() -> &'static str;

    fn column_aliases(prefix: Option<&str>) -> Vec<String> {
        let pre = if prefix.is_some() {
            format!("{}.{}", prefix.as_ref().unwrap(), Self::table_name())
        } else {
            Self::table_name().to_string()
        };

        Self::columns()
            .iter()
            .map(|c| format!("{0}.{2} as '{1}.{2}'", Self::table_name(), pre, c))
            .collect()
    }
}
