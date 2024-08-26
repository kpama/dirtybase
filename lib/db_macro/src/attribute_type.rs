use std::collections::HashMap;

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
                name: String::new(),
                optional: false,
                the_type: String::new(),
            },
            "from" => Self::FromHandler(String::new()),
            "into" => Self::IntoHandler(String::new()),
            _ => Self::ColName {
                name: String::new(),
                optional: false,
                the_type: String::new(),
            },
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct DirtybaseAttributes {
    pub(crate) name: String,
    pub(crate) optional: bool,
    pub(crate) the_type: String,
    pub(crate) is_vec: bool,
    pub(crate) flatten: bool,
    pub(crate) embeded: bool,
    pub(crate) from_handler: String,
    pub(crate) has_custom_from_handler: bool,
    pub(crate) skip_select: bool, // Don't include the column in the list of columns when selecting
    pub(crate) into_handler: String,
    pub(crate) has_custom_into_handler: bool,
    pub(crate) skip_insert: bool, // Don's include the column in the list of columns when inserting
    pub(crate) relation: RelType,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) enum RelType {
    HasOne {
        attribute: RelationAttribute,
    },
    BelongsTo {
        attribute: RelationAttribute,
    },
    HasMany {
        attribute: RelationAttribute,
    },
    HasOneThrough {
        attribute: RelationAttribute,
    },
    HasManyThrough {
        attribute: RelationAttribute,
    },
    BelongsToMany {
        attribute: RelationAttribute,
    },
    #[default]
    None,
}

impl RelType {
    pub(crate) fn new(mut attribute: HashMap<String, String>) -> Self {
        let name = attribute.remove("kind").unwrap_or_default();
        // TODO: Make sure the attributes are set correctly or fallback the defaults
        match name.as_str() {
            "has_one" => Self::HasOne {
                attribute: attribute.into(),
            },
            "belongs_to" => Self::BelongsTo {
                attribute: attribute.into(),
            },
            "belongs_to_many" => Self::BelongsToMany {
                attribute: attribute.into(),
            },
            "has_many" => Self::HasMany {
                attribute: attribute.into(),
            },
            "has_one_through" => Self::HasOneThrough {
                attribute: attribute.into(),
            },
            "has_many_through" => Self::HasManyThrough {
                attribute: attribute.into(),
            },
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelationAttribute {
    pub(crate) foreign_key: Option<String>,
    pub(crate) foreign_tbl: Option<String>,
    pub(crate) local_key: Option<String>,
    pub(crate) this_tbl: Option<String>,
    pub(crate) final_key: Option<String>,
    pub(crate) final_tbl: Option<String>,
    pub(crate) through_tbl: Option<String>,
    pub(crate) through_key: Option<String>,
    pub(crate) through_final_key: Option<String>,
    pub(crate) pivot_key: Option<String>,
    pub(crate) pivot_tbl: Option<String>,
}

impl Into<RelationAttribute> for HashMap<String, String> {
    fn into(mut self) -> RelationAttribute {
        RelationAttribute {
            foreign_key: self.remove("foreign_key"),
            foreign_tbl: self.remove("foreign_tbl"),
            local_key: self.remove("local_key"),
            this_tbl: self.remove("this_tbl"),
            final_key: self.remove("final_key"),
            final_tbl: self.remove("final_tbl"),
            through_tbl: self.remove("through_tbl"),
            through_key: self.remove("through_key"),
            through_final_key: self.remove("through_final_key"),
            pivot_key: self.remove("pivot_key"),
            pivot_tbl: self.remove("pivot_tbl"),
        }
    }
}
