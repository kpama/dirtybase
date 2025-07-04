use std::collections::HashMap;

use syn::DeriveInput;

use crate::relationship::{belongs_to, has_many, has_one, has_one_through};

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
    pub(crate) embedded: bool,
    pub(crate) from_handler: String,
    pub(crate) has_custom_from_handler: bool,
    pub(crate) skip_select: bool, // Don't include the column in the list of columns when selecting
    pub(crate) into_handler: String,
    pub(crate) has_custom_into_handler: bool,
    pub(crate) skip_insert: bool, // Don't include the column in the list of columns when inserting
    pub(crate) relation: Option<RelType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelationAttribute {
    pub(crate) foreign_key: Option<String>,
    // pub(crate) foreign_tbl: Option<String>,
    pub(crate) local_key: Option<String>,
    // pub(crate) this_tbl: Option<String>,
    // pub(crate) final_key: Option<String>,
    // pub(crate) final_tbl: Option<String>,
    // pub(crate) through_tbl: Option<String>,
    pub(crate) through_key: Option<String>,
    // pub(crate) through_final_key: Option<String>,
    // pub(crate) pivot_key: Option<String>,
    pub(crate) pivot_through_key: Option<String>,
    // pub(crate) pivot_tbl: Option<String>,
    pub(crate) pivot: Option<String>,
}

impl From<HashMap<String, String>> for RelationAttribute {
    fn from(mut val: HashMap<String, String>) -> Self {
        RelationAttribute {
            foreign_key: val.remove("foreign_key"),
            // foreign_tbl: val.remove("foreign_tbl"),
            local_key: val.remove("local_key"),
            // this_tbl: val.remove("this_tbl"),
            // final_key: val.remove("final_key"),
            // final_tbl: val.remove("final_tbl"),
            // through_tbl: val.remove("through_tbl"),
            through_key: val.remove("through_key"),
            // through_final_key: val.remove("through_final_key"),
            // pivot_key: val.remove("pivot_key"),
            pivot_through_key: val.remove("pivot_through_key"),
            // pivot_tbl: val.remove("pivot_tbl"),
            pivot: val.remove("pivot"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RelType {
    HasOne { attribute: RelationAttribute },
    BelongsTo { attribute: RelationAttribute },
    HasMany { attribute: RelationAttribute },
    HasOneThrough { attribute: RelationAttribute },
    HasManyThrough { attribute: RelationAttribute },
    BelongsToMany { attribute: RelationAttribute },
}

impl RelType {
    pub(crate) fn new(
        mut attribute: HashMap<String, String>,
        field: &syn::Field,
        input: &DeriveInput,
    ) -> Option<Self> {
        let name = attribute.remove("kind").unwrap_or_default();
        // TODO: Make sure the attributes are set correctly or fallback to the defaults
        match name.as_str() {
            "has_one" => Some(Self::HasOne {
                attribute: has_one::build_attribute(attribute, field, input),
            }),
            "belongs_to" => Some(Self::BelongsTo {
                attribute: belongs_to::build_attribute(attribute, field, input),
            }),
            "belongs_to_many" => Some(Self::BelongsToMany {
                attribute: attribute.into(),
            }),
            "has_many" => Some(Self::HasMany {
                attribute: has_many::build_attribute(attribute, field, input),
            }),
            "has_one_through" => Some(Self::HasOneThrough {
                attribute: has_one_through::build_attribute(attribute, field, input),
            }),
            "has_many_through" => Some(Self::HasManyThrough {
                attribute: attribute.into(),
            }),
            _ => None,
        }
    }
}
