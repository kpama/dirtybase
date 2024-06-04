use crate::{attribute_type::DirtybaseAttributes, helpers::pluck_id_column};
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::DeriveInput;

mod repo_basic_query;
mod type_query;

pub(crate) fn generate_entity_repo(
    columns: &Vec<(String, DirtybaseAttributes)>,
    base_name: &Ident,
    table_name: &str,
    input: &DeriveInput,
) -> proc_macro2::TokenStream {
    let struct_name = format_ident!("{}Repository", base_name);
    let the_table_name = table_name.to_string();
    let mut methods = Vec::new();
    let id_column = pluck_id_column(input);

    // types queries
    methods = repo_basic_query::generate(columns, methods, base_name, &id_column);
    methods = type_query::string_query::generate(columns, methods, base_name);
    methods = type_query::nullable_query::generate(columns, methods, base_name);
    methods = type_query::number_query::generate(columns, methods, base_name);

    quote! {
      pub struct  #struct_name {
        manager: dirtybase_db::base::manager::Manager,
        table: String,
      }

      impl #struct_name {
        pub fn new(manager: dirtybase_db::base::manager::Manager) -> Self {
            Self {
              manager,
              table: #the_table_name.to_string()
             }
        }

        pub fn manager(&self) -> &dirtybase_db::base::manager::Manager {
          &self.manager
        }

        pub fn builder(&self)-> dirtybase_db::base::query::EntityQueryBuilder<#base_name>{
          self.manager.query_builder(&self.table)
        }

        #(#methods)*
      }


    }
}
