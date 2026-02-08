use std::collections::HashMap;

use syn::{DeriveInput, Meta};

use crate::relationship::{
    belongs_to, has_many, has_many_through, has_one, has_one_through, morph_many, morph_one,
};

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

#[derive(Debug, Clone)]
pub(crate) struct TableAttribute {
    pub(crate) table_name: String,
    pub(crate) id_field: String,
    pub(crate) id_column: String,
    pub(crate) foreign_name: String,
    pub(crate) no_timestamp: bool,
    pub(crate) no_soft_delete: bool,
    pub(crate) created_at_col: String,
    pub(crate) updated_at_col: String,
    pub(crate) deleted_at_col: String,
}

impl Default for TableAttribute {
    fn default() -> Self {
        Self {
            table_name: String::new(),
            id_field: "id".to_string(),
            id_column: "id".to_string(),
            foreign_name: String::new(),
            no_timestamp: false,
            no_soft_delete: false,
            created_at_col: "created_at".to_string(),
            updated_at_col: "updated_at".to_string(),
            deleted_at_col: "deleted_at".to_string(),
        }
    }
}

impl From<&DeriveInput> for TableAttribute {
    fn from(input: &DeriveInput) -> Self {
        let table_name = cruet::case::to_table_case(&input.ident.clone().to_string());
        let mut value = Self::default();

        value.table_name = table_name;

        for attr in &input.attrs {
            if let Meta::List(the_list) = &attr.meta
                && the_list.path.is_ident("dirty")
            {
                let mut walker = the_list.tokens.clone().into_iter();
                while let Some(arg) = walker.next() {
                    if arg.to_string() == "no_timestamp" {
                        value.no_timestamp = true;
                    }

                    if arg.to_string() == "no_soft_delete" {
                        value.no_soft_delete = true;
                    }

                    if arg.to_string() == "created_at" {
                        _ = walker.next();
                        if let Some(name) = walker.next() {
                            value.created_at_col = name.to_string().replace('\"', "");
                        }
                    }

                    if arg.to_string() == "updated_at" {
                        _ = walker.next();
                        if let Some(name) = walker.next() {
                            value.updated_at_col = name.to_string().replace('\"', "");
                        }
                    }

                    if arg.to_string() == "deleted_at" {
                        _ = walker.next();
                        if let Some(name) = walker.next() {
                            value.deleted_at_col = name.to_string().replace('\"', "");
                        }
                    }

                    if arg.to_string() == "table" {
                        _ = walker.next();
                        if let Some(name) = walker.next() {
                            value.table_name = name.to_string().replace('\"', "");
                        }
                    }
                    if arg.to_string() == "id" {
                        _ = walker.next();
                        if let Some(name) = walker.next() {
                            value.id_field = name.to_string().replace('\"', "");
                        }
                    }
                    if arg.to_string() == "id_column" {
                        _ = walker.next();
                        if let Some(name) = walker.next() {
                            value.id_column = name.to_string().replace('\"', "");
                        }
                    }

                    if arg.to_string() == "foreign_name" {
                        _ = walker.next();
                        if let Some(name) = walker.next() {
                            value.foreign_name = name.to_string().replace('\"', "");
                        }
                    }
                }
            }
        }

        if value.foreign_name.is_empty() {
            value.foreign_name = format!(
                "{}_{}",
                cruet::string::singularize::to_singular(&value.table_name),
                &value.id_column
            );
        }

        value
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
    pub(crate) foreign_col: Option<String>,
    pub(crate) local_col: Option<String>,
    pub(crate) through_col: Option<String>,
    pub(crate) pivot_through_col: Option<String>,
    pub(crate) pivot: Option<String>,
    pub(crate) morph_name: Option<String>,
    pub(crate) morph_type: Option<String>,
    pub(crate) morph_type_col: Option<String>,
    pub(crate) no_soft_delete: bool,
}

impl From<HashMap<String, String>> for RelationAttribute {
    fn from(mut val: HashMap<String, String>) -> Self {
        RelationAttribute {
            foreign_col: val.remove("foreign_col"),
            local_col: val.remove("local_col"),
            through_col: val.remove("through_col"),
            pivot_through_col: val.remove("pivot_through_col"),
            pivot: val.remove("pivot"),
            morph_name: val.remove("morph_name"),
            morph_type: val.remove("morph_type"),
            morph_type_col: val.remove("morph_type_col"),
            no_soft_delete: val.remove("no_soft_delete").is_some(),
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
    // MorphTo { attribute: RelationAttribute },
    MorphOne { attribute: RelationAttribute },
    MorphMany { attribute: RelationAttribute },
}

impl RelType {
    pub(crate) fn new(
        mut attribute: HashMap<String, String>,
        field: &syn::Field,
        input: &DeriveInput,
    ) -> Option<Self> {
        let name = attribute.remove("kind").unwrap_or_default();
        // TODO: Make sure the attributes are set correctly or fallback to the defaults
        match name.to_lowercase().as_str() {
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
                attribute: has_many_through::build_attribute(attribute, field, input),
            }),
            "morph_one" => Some(Self::MorphOne {
                attribute: morph_one::build_attribute(attribute, field, input),
            }),
            "morph_many" => Some(Self::MorphMany {
                attribute: morph_many::build_attribute(attribute, field, input),
            }),
            _ => None,
        }
    }
}
