use std::collections::HashMap;

use syn::{DeriveInput, Meta};

use crate::relationship::{
    belongs_to, has_many, has_many_through, has_one, has_one_through, morph_many, morph_one,
};

#[derive(Debug, Clone)]
pub(crate) struct TableAttribute {
    pub(crate) table_name: String,
    pub(crate) id_field: String,
    pub(crate) id_column: String,
    pub(crate) foreign_name: String,
    pub(crate) id_incrementing: bool,
    pub(crate) timestamp: bool,
    pub(crate) soft_deletable: bool,
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
            id_incrementing: true,
            timestamp: false,
            soft_deletable: false,
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
                    if arg.to_string() == "timestamp" || arg.to_string() == "timestampable" {
                        value.timestamp = true;
                    }

                    if arg.to_string() == "id_not_auto" {
                        value.id_incrementing = false;
                    }

                    if arg.to_string() == "soft_delete" || arg.to_string() == "soft_deletable" {
                        value.soft_deletable = true;
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
    pub(crate) soft_deletable: bool,
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
            soft_deletable: val.remove("soft_deletable").is_some(),
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
        if name.is_empty() {
            return None;
        }

        match name.to_lowercase().as_str() {
            "has_one" => Some(Self::HasOne {
                attribute: has_one::build_attribute(attribute, field, input),
            }),
            "belongs_to" => Some(Self::BelongsTo {
                attribute: belongs_to::build_attribute(attribute, field, input),
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
            _ => panic!("unknown relation kind: {}", name),
        }
    }
}
