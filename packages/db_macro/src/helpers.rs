use crate::{
    attribute_type::{DirtybaseAttributes, TableAttribute},
    relationship::process_relation_attribute,
};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use std::collections::HashMap;
use syn::{Data, DeriveInput, GenericArgument, Meta, MetaList, PathArguments, TypePath};

pub(crate) fn pluck_columns(
    input: &DeriveInput,
    tbl_attr: &mut TableAttribute,
) -> HashMap<String, DirtybaseAttributes> {
    let mut columns = HashMap::new();

    if let Data::Struct(data) = &input.data {
        if let syn::Fields::Named(fields) = &data.fields {
            for a_field in fields.named.iter() {
                if let Some(a_col) = get_real_column_name(a_field, input) {
                    if tbl_attr.id_field == a_col.0 && tbl_attr.id_column != a_col.1.name {
                        tbl_attr.id_column = a_col.1.name.clone();
                    }

                    columns.insert(a_col.0, a_col.1);
                }
            }
        }
    }

    columns
}

pub(crate) fn get_real_column_name(
    field: &syn::Field,
    input: &DeriveInput,
) -> Option<(String, DirtybaseAttributes)> {
    let name = field.ident.as_ref().unwrap().to_string();

    let mut dirty_attribute = DirtybaseAttributes {
        from_handler: format!("from_column_for_{}", &name),
        into_handler: format!("into_column_for_{}", &name),
        name: name.clone(),
        ..DirtybaseAttributes::default()
    };

    let mut include_column = false;

    if !field.attrs.is_empty() {
        for attr in &field.attrs {
            if let Meta::List(the_list) = &attr.meta {
                include_column =
                    field_attributes(field, Some(the_list), &mut dirty_attribute, input);
            }
        }
    } else {
        include_column = field_attributes(field, None, &mut dirty_attribute, input);
    }

    if include_column {
        Some((name, dirty_attribute))
    } else {
        None
    }
}

pub(crate) fn field_attributes(
    field: &syn::Field,
    metalist: Option<&MetaList>,
    dirty_attribute: &mut DirtybaseAttributes,
    input: &DeriveInput,
) -> bool {
    let mut include = true;
    let name = field.ident.as_ref().unwrap().to_string();

    if !name.is_empty() {
        dirty_attribute.name = name;
    }

    if let Some(meta) = metalist {
        if meta.path.is_ident("dirty") {
            let walker = meta.tokens.clone().into_iter();
            include = attribute_to_attribute_type(walker, field, dirty_attribute, input);
        } else {
            make_column_name_attribute_type(field, dirty_attribute);
        }
    } else {
        make_column_name_attribute_type(field, dirty_attribute);
    }

    include
}

pub(crate) fn attribute_to_attribute_type(
    mut walker: proc_macro2::token_stream::IntoIter,
    field: &syn::Field,
    dirty_attribute: &mut DirtybaseAttributes,
    input: &DeriveInput,
) -> bool {
    let mut include = true;

    make_column_name_attribute_type(field, dirty_attribute);

    if let Some(key) = walker.next() {
        match key.to_string().as_str() {
            "rel" => {
                if let Some(tree) = walker.next() {
                    include = process_relation_attribute(
                        field,
                        dirty_attribute,
                        tree.into_token_stream(),
                        input,
                    );
                }
            }
            "col" => {
                _ = walker.next();
                dirty_attribute.name = walker.next().unwrap().to_string().replace('\"', "");
            }
            "from" => {
                _ = walker.next();
                let from_handler = walker.next().unwrap().to_string().replace('\"', "");
                dirty_attribute.from_handler = from_handler;
                dirty_attribute.has_custom_from_handler = true;
            }
            "into" => {
                _ = walker.next();
                let into_handler = walker.next().unwrap().to_string().replace('\"', "");
                dirty_attribute.into_handler = into_handler;
                dirty_attribute.has_custom_into_handler = true;
            }
            "skip_select" => {
                dirty_attribute.skip_select = true;
            }
            "skip_insert" => {
                dirty_attribute.skip_insert = true;
            }
            "skip" => {
                include = false;
            }
            "flatten" => {
                dirty_attribute.flatten = true;
            }
            "embedded" => {
                dirty_attribute.embedded = true;
            }
            _ => (),
        };

        if let Some(x) = walker.next() {
            if x.to_string() == "," {
                attribute_to_attribute_type(walker, field, dirty_attribute, input);
            }
        }
    }

    include
}

pub(crate) fn make_column_name_attribute_type(
    field: &syn::Field,
    dirty_attribute: &mut DirtybaseAttributes,
) {
    if let syn::Type::Path(ref p) = field.ty {
        walk_and_find_type(p, dirty_attribute);
    }
}

fn walk_and_find_type(p: &TypePath, dirty_attribute: &mut DirtybaseAttributes) {
    if &p.path.segments[0].ident.to_string() == "Option" {
        dirty_attribute.optional = true;

        if let PathArguments::AngleBracketed(a) = &p.path.segments[0].arguments {
            if let GenericArgument::Type(syn::Type::Path(p)) = &a.args[0] {
                if let Some(f) = p.path.get_ident() {
                    dirty_attribute.the_type = f.to_string();
                } else {
                    walk_and_find_type(p, dirty_attribute);
                }
            }
        }
    } else {
        let name = p.path.segments[0].ident.to_string();
        if name == "Vec" {
            if let PathArguments::AngleBracketed(a) = &p.path.segments[0].arguments {
                if let GenericArgument::Type(syn::Type::Path(p)) = &a.args[0] {
                    dirty_attribute.the_type = p.path.segments[0].ident.to_string();
                    dirty_attribute.is_vec = true;
                }
            }
        } else {
            dirty_attribute.the_type = p.path.segments[0].ident.to_string();
        }
    }
}

pub(crate) fn pluck_names(
    columns_attributes: &HashMap<String, DirtybaseAttributes>,
) -> Vec<String> {
    columns_attributes
        .iter()
        .filter(|c| !c.1.skip_select)
        .filter(|c| !c.1.relation.is_some())
        .filter(|c| !c.1.flatten)
        .map(|c| c.1.name.clone())
        .collect::<Vec<String>>()
}

pub(crate) fn names_of_from_cv_handlers(
    columns_attributes: &HashMap<String, DirtybaseAttributes>,
    table_name: &String,
) -> Vec<TokenStream> {
    columns_attributes
        .iter()
        // .filter(|c| c.1.relation.is_none())
        .map(|item| {
            let struct_field = format_ident!("{}", &item.0);
            let column = if *item.0 == item.1.name {
                item.0
            } else {
                &item.1.name
            };
            let handler = format_ident!("{}", &item.1.from_handler);

            if item.1.flatten {
                let the_type = format_ident!("{}", &item.1.the_type);
                return quote! {
                    #struct_field:#the_type::from_column_value(cv.clone()).expect("could not flatten")
                };
            }

                quote! {
                    #struct_field: Self::#handler(if cv.contains_key(#column) {
                        cv.get(#column)
                    } else {
                        match cv.get(#table_name) {
                          Some(::dirtybase_contract::db_contract::field_values::FieldValue::Object(c)) => {
                               c.get(#column).clone()
                          },
                          _ => None
                        }
                    } )
                }
        })
        .collect()
}

pub(crate) fn spread_default(
    columns_attributes: &HashMap<String, DirtybaseAttributes>,
    input: &DeriveInput,
) -> TokenStream {
    let length = match &input.data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => fields.named.iter().len(),
            _ => 0,
        },
        _ => 0,
    };

    if length > 0 && length != columns_attributes.len() {
        quote! {
            ..Self::default()
        }
    } else {
        quote! {
            // Nothing do do
        }
    }
}

pub(crate) fn build_from_handlers(
    columns_attributes: &HashMap<String, DirtybaseAttributes>,
) -> Vec<proc_macro2::TokenStream> {
    let mut built: Vec<proc_macro2::TokenStream> = Vec::new();
    for item in columns_attributes.iter() {
        let returns = format_ident!("{}", &item.1.the_type);
        let fn_name = format_ident!("{}", &item.1.from_handler);

        if item.1.has_custom_from_handler {
            continue;
        }

        built.push(
                    if item.1.optional {
                        if item.1.is_vec {
                        quote! {
                            pub fn #fn_name <'a>(field: Option<&'a ::dirtybase_contract::db_contract::field_values::FieldValue>) -> Option<Vec<#returns>> {
                                ::dirtybase_contract::db_contract::field_values::FieldValue::from_ref_option_into_option(field)
                            }
                        }
                        } else {
                            quote! {
                                pub fn #fn_name <'a>(field: Option<&'a ::dirtybase_contract::db_contract::field_values::FieldValue>) -> Option<#returns> {
                                ::dirtybase_contract::db_contract::field_values::FieldValue::from_ref_option_into_option(field)
                                }
                            }
                        }
                    } else if item.1.is_vec {
                            quote! {
                                pub fn #fn_name <'a> (field: Option<&'a ::dirtybase_contract::db_contract::field_values::FieldValue>) -> Vec<#returns> {
                                    ::dirtybase_contract::db_contract::field_values::FieldValue::from_ref_option_into(field)
                                }
                            }
                        }
                        else {
                            quote! {
                                pub fn #fn_name <'a> (field: Option<&'a ::dirtybase_contract::db_contract::field_values::FieldValue>) -> #returns {
                                    ::dirtybase_contract::db_contract::field_values::FieldValue::from_ref_option_into(field)
                                }
                        }
                    });
    }

    built
}

// TODO: implement "into handler"
pub(crate) fn build_into_handlers(
    columns_attributes: &HashMap<String, DirtybaseAttributes>,
) -> Vec<proc_macro2::TokenStream> {
    let mut built: Vec<proc_macro2::TokenStream> = Vec::new();

    for item in columns_attributes.iter() {
        let fn_name = format_ident!("{}", &item.1.into_handler);
        let struct_field = format_ident!("{}", &item.0);

        if item.1.has_custom_into_handler || item.1.relation.is_some() {
            continue;
        }

        built.push(if item.1.optional {
            if item.1.embedded {
                quote! {
                    pub fn #fn_name(&self) ->Option<::dirtybase_contract::db_contract::field_values::FieldValue> {
                        if let Some(value) = &self.#struct_field {
                            Some(value.into_embeddable())
                        } else {
                            None
                        }
                    }
                }
            } else {
                quote! {
                    pub fn #fn_name(&self) ->Option<::dirtybase_contract::db_contract::field_values::FieldValue> {
                        if let Some(value) = &self.#struct_field {
                            Some(value.clone().into())
                        } else {
                            None
                        }
                    }
                }
            }
        } else if item.1.embedded {
            quote! {
                pub fn #fn_name(&self) ->Option<::dirtybase_contract::db_contract::field_values::FieldValue> {
                     Some(self.#struct_field.into_embeddable())
                    }
                }
            } else {
                quote! {
                    pub fn #fn_name(&self) ->Option<::dirtybase_contract::db_contract::field_values::FieldValue> {
                        Some(self.#struct_field.clone().into())
                    }
                }
        });
    }

    built
}

pub(crate) fn build_into_for_calls(
    columns_attributes: &HashMap<String, DirtybaseAttributes>,
) -> Vec<proc_macro2::TokenStream> {
    let mut built: Vec<proc_macro2::TokenStream> = Vec::new();
    for item in columns_attributes.iter() {
        let name = &item.1.name;
        let method_name = format_ident!("{}", &item.1.into_handler);

        if item.1.skip_insert || item.1.relation.is_some() {
            continue;
        }

        if item.1.flatten {
            let prop_name = format_ident!("{}", &item.1.name);
            built.push(quote! {
                merge_column_value(::dirtybase_contract::db_contract::types::ToColumnAndValue::to_column_value(&self.#prop_name).expect("could not flatten type"))
            });
            continue;
        }

        built.push(quote! {
            try_to_insert_field_value(#name, self.#method_name())
        });
    }

    built
}

pub(crate) fn pluck_attributes(
    input: &DeriveInput,
) -> (TableAttribute, HashMap<String, DirtybaseAttributes>) {
    let mut tbl_attr = TableAttribute::from(input);
    let columns = pluck_columns(input, &mut tbl_attr);
    (tbl_attr, columns)
}

pub(crate) fn build_entity_hash_method(tbl_attr: &TableAttribute) -> TokenStream {
    let id_field = format_ident!("{}", &tbl_attr.id_field);
    quote! {
        fn entity_hash(&self) -> u64 {
            let mut s = ::std::hash::DefaultHasher::new();
            ::std::hash::Hash::hash(&self.#id_field, &mut s);
            ::std::hash::Hasher::finish(&s)
        }
    }
}

pub(crate) fn build_special_column_methods(
    tbl_attr: &TableAttribute,
) -> Vec<proc_macro2::TokenStream> {
    let mut tokens = Vec::new();

    // table name
    let tbl_name = &tbl_attr.table_name;
    tokens.push(quote! {
        fn table_name() -> &'static str {
            #tbl_name
        }
    });

    // foreign id
    let foreign_id = &tbl_attr.foreign_name;
    tokens.push(quote! {
        fn foreign_id_column() -> &'static str {
            #foreign_id
        }
    });

    // id column
    if tbl_attr.id_column != "id" {
        let id = &tbl_attr.id_column;
        tokens.push(quote! {
            fn id_column() -> &'static str {
                #id
            }
        });
    }

    // timestamps
    if !tbl_attr.no_timestamp {
        let created_at = &tbl_attr.created_at_col;
        let updated_at = &tbl_attr.updated_at_col;
        tokens.push(quote! {
            fn created_at_column() -> Option<&'static str> {
                Some(#created_at)
            }
        });

        tokens.push(quote! {
            fn updated_at_column() -> Option<&'static str> {
                Some(#updated_at)
            }
        });
    }

    // soft delete
    if !tbl_attr.no_softdelete {
        let name = &tbl_attr.deleted_at_col;
        tokens.push(quote! {
            fn deleted_at_column() -> Option<&'static str> {
                Some(#name)
            }
        });
    }

    tokens
}

/// Builds static method for each field/prop in the struct
/// that corresponds to a table column
pub(crate) fn build_prop_column_names_getter(
    columns_attributes: &HashMap<String, DirtybaseAttributes>,
) -> Vec<proc_macro2::TokenStream> {
    let mut built: Vec<proc_macro2::TokenStream> = Vec::new();

    for item in columns_attributes.iter() {
        if item.1.relation.is_some() {
            continue;
        }

        let fn_name = format_ident!("col_name_for_{}", item.0);
        let col_name = &item.1.name;
        built.push(quote! {
                pub fn #fn_name() -> &'static str {
                    #col_name
                }
        })
    }

    built
}
