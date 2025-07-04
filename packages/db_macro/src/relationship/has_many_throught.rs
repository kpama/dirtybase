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

    if let Some(RelType::HasManyThrough { attribute }) = &attr.relation {
        let pivot_type = if let Some(p) = &attribute.pivot {
            format_ident!("{}", p)
        } else {
            std::println!("pivot type not specified for: {}", name);
            return;
        };
        // parent key
        let mut parent_key = quote! {<#parent as ::dirtybase_contract::db_contract::table_entity::TableEntityTrait>::id_column()};
        // foreign key
        let mut foreign_key = quote! { <#parent as ::dirtybase_contract::db_contract::table_entity::TableEntityTrait>::foreign_id_column() };
        // pivot fk key
        let mut pivot_through_key = quote! { <#foreign_type as ::dirtybase_contract::db_contract::table_entity::TableEntityTrait>::foreign_id_column() };
        let mut through_key = quote! { <#foreign_type as ::dirtybase_contract::db_contract::table_entity::TableEntityTrait>::id_column() };

        // parent key
        if let Some(field) = &attribute.local_key {
            parent_key = quote! { #field };
        }

        // foreign key
        if let Some(field) = &attribute.foreign_key {
            foreign_key = quote! { #field };
        }

        // pivot fk key
        if let Some(field) = &attribute.pivot_through_key {
            pivot_through_key = quote! { #field };
        }

        if let Some(field) = &attribute.through_key {
            through_key = quote! { #field };
        }

        let token = quote! {
            pub fn #method_name(&mut self,) -> &mut Self {
                let name = #name.to_string();
                if !self.eager.contains(&name) {
                    self.builder.inner_join_table_and_select::<#parent, #pivot_type>(#parent_key, #foreign_key, None);
                    self.builder.inner_join_table_and_select::<#pivot_type, #foreign_type>(#pivot_through_key, #through_key, None);
                    self.eager.push(name);
                }
                self
            }
        };

        list.insert(method_name_st, token);
    }
}
