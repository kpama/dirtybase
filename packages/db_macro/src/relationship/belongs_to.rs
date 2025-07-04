use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

use crate::attribute_type::RelType;
use crate::attribute_type::{DirtybaseAttributes, RelationAttribute};

pub(crate) fn build_attribute(
    attr: HashMap<String, String>,
    _field: &syn::Field,
    _input: &DeriveInput,
) -> RelationAttribute {
    let attribute = RelationAttribute::from(attr);
    //
    attribute
}

pub(crate) fn generate_join_method(
    attr: &DirtybaseAttributes,
    input: &DeriveInput,
    list: &mut HashMap<String, TokenStream>,
) {
    // method
    let name = &attr.name;
    let method_name_st = format!("with_{}", name);
    let method_name = format_ident!("{}", &method_name_st);
    let parent = format_ident!("{}", &input.ident);
    let foreign_type = format_ident!("{}", attr.the_type);

    // parent key
    let mut parent_key = quote! {<#foreign_type as ::dirtybase_contract::db_contract::table_entity::TableEntityTrait>::foreign_id_column()};
    // foreign key
    let mut foreign_key = quote! { <#foreign_type  as ::dirtybase_contract::db_contract::table_entity::TableEntityTrait>::id_column() };

    if let Some(RelType::BelongsTo { attribute }) = &attr.relation {
        // parent key
        if let Some(field) = &attribute.local_key {
            parent_key = quote! { #field };
        }

        // foreign key
        if let Some(field) = &attribute.foreign_key {
            foreign_key = quote! { #field };
        }
    }

    let token = quote! {
        pub fn #method_name(&mut self,) -> &mut Self {
            let name = #name.to_string();
            if !self.eager.contains(&name) {
                self.builder.inner_join_table_and_select::<#parent,#foreign_type>(#parent_key, #foreign_key, None);
                self.eager.push(name);
            }
            self
        }
    };

    list.insert(method_name_st, token);
}

pub(crate) fn append_result_collection(
    attr: &DirtybaseAttributes,
    list: &mut HashMap<String, TokenStream>,
) {
    let name = &attr.name;
    let foreign_type = format_ident!("{}", attr.the_type);
    let map_name_st = format!("{}_map", name);
    let map_name = format_ident!("{}", &map_name_st);
    let is_eager = format_ident!("are_{}_eager", name);

    let token = quote! {
        let mut #map_name: ::std::collections::HashMap::<u64,#foreign_type> = ::std::collections::HashMap::new();
        let #is_eager = self.eager.contains(&#name.to_string());
    };

    list.insert(map_name_st, token);
}

pub(crate) fn build_row_processor(
    attr: &DirtybaseAttributes,

    list: &mut HashMap<String, TokenStream>,
) {
    let name = &attr.name;
    let is_eager = format_ident!("are_{}_eager", name);
    let map_name_st = format!("{}_map", name);
    let map_name = format_ident!("{}", &map_name_st);
    let foreign_type = format_ident!("{}", attr.the_type);

    let token = quote! {
       //
       if #is_eager {
            if let Some(entity) = #foreign_type::from_struct_column_value(row,
                 Some(<#foreign_type as ::dirtybase_contract::db_contract::table_entity::TableEntityTrait>::table_name())) {
                #map_name.insert(row_hash ,entity);
            }
       }
    };

    list.insert(map_name_st, token);
}

pub(crate) fn build_entity_append(
    attr: &DirtybaseAttributes,
    list: &mut HashMap<String, TokenStream>,
) {
    let name = &attr.name;
    let is_eager = format_ident!("are_{}_eager", name);
    let map_name_st = format!("{}_map", name);
    let map_name = format_ident!("{}", &map_name_st);
    let name_ident = format_ident!("{}", name);

    let body = if attr.optional {
        quote! {
                row_entity.#name_ident = Some(parent);
        }
    } else {
        quote! {
                row_entity.#name_ident = parent;
        }
    };

    let token = quote! {
        //
        if #is_eager {
            //
            if let Some(parent) = #map_name.get(&row_hash).cloned() {
                #body
            }
        }
    };

    list.insert(map_name_st, token);
}
