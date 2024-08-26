use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::attribute_type::{DirtybaseAttributes, RelationAttribute};

pub(crate) fn generate_method(
    column: &DirtybaseAttributes,
    attribute: &RelationAttribute,
    _base_name: &Ident,
    table_name: &str,
    prop_name: &Ident,
    method_name: &Ident,
    the_type: &Ident,
    postfix: &str,
    default_id_column: &str,
    default_fk_column: &str,
) -> TokenStream {
    let parent_model = format_ident!("{}{}", &column.the_type, postfix);

    let id_field = format_ident!(
        "{}",
        attribute
            .local_key
            .clone()
            .unwrap_or_else(|| default_id_column.to_string())
    );

    let tbl_name = if attribute.foreign_tbl.is_some() {
        let name = attribute.foreign_tbl.as_ref().unwrap();
        quote! { #name }
    } else {
        quote! {<#the_type as ::dirtybase_contract::db::TableEntityTrait>::table_name()}
    };

    let foreign_id = if attribute.foreign_key.is_some() {
        let name = attribute.foreign_key.as_ref().unwrap();
        quote! { #name }
    } else {
        quote! {<#the_type as ::dirtybase_contract::db::TableEntityTrait>::id_column().unwrap()}
    };

    quote! {
      pub fn #method_name(&mut self) -> #parent_model {

        if self.#prop_name.is_none() {
          let mut builder = ::dirtybase_contract::db::base::query::QueryBuilder::new(
                  #tbl_name,
                  ::dirtybase_contract::db::base::query::QueryAction::Query { columns: None },
            );

            // lazy loading
            if  self.fetched {
                builder.eq(#foreign_id, self.entity.#id_field.clone());
            }

            self.#prop_name= Some(builder);
        }

        #parent_model::new_with_builder(std::sync::Arc::clone(&self.manager),self.#prop_name.clone().unwrap())
      }
    }
}
