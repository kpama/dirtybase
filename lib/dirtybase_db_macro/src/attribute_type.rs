#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AttributeType {
    ColName {
        name: String,
        optional: bool,
        the_type: String,
    },
    FromHandler(String),
    IntoHandler(String),
}

impl From<String> for AttributeType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "col" => Self::ColName {
                name: "".into(),
                optional: false,
                the_type: "".into(),
            },
            "from" => Self::FromHandler("".into()),
            "into" => Self::IntoHandler("".into()),
            _ => Self::ColName {
                name: "".into(),
                optional: false,
                the_type: "".into(),
            },
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct DirtybaseAttributes {
    pub(crate) name: String,
    pub(crate) optional: bool,
    pub(crate) the_type: String,
    pub(crate) from_handler: String,
    pub(crate) has_custom_from_handler: bool,
    pub(crate) skip_select: bool, // Don't include the column in the list of columns
    pub(crate) into_handler: String,
    pub(crate) has_custom_into_handler: bool,
    pub(crate) skip_insert: bool,
}
