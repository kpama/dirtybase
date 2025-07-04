use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

use crate::{
    attribute_type::{DirtybaseAttributes, RelType},
    relationship::{belongs_to, has_many, has_many_through, has_one, has_one_through},
};

pub fn build_entity_repo(
    input: &DeriveInput,
    columns_attributes: &HashMap<String, DirtybaseAttributes>,
) -> TokenStream {
    //
    let ident = input.ident.clone();
    let repo_name = format_ident!("{}Repo", &input.ident);
    let mut with_methods = HashMap::<String, TokenStream>::new();
    let mut collections = HashMap::<String, TokenStream>::new();
    let mut row_processors = HashMap::<String, TokenStream>::new();
    let mut entity_appends = HashMap::<String, TokenStream>::new();

    for (_, attr) in columns_attributes {
        match &attr.relation {
            Some(RelType::HasOne { attribute: _ }) => {
                has_one::generate_join_method(attr, input, &mut with_methods);
                has_one::append_result_collection(attr, &mut collections);
                has_one::build_row_processor(attr, &mut row_processors);
                has_one::build_entity_append(attr, &mut entity_appends);
            }
            Some(RelType::HasMany { attribute: _ }) => {
                has_many::generate_join_method(attr, input, &mut with_methods);
                has_many::append_result_collection(attr, &mut collections);
                has_many::build_row_processor(attr, &mut row_processors);
                has_many::build_entity_append(attr, &mut entity_appends);
            }
            Some(RelType::BelongsTo { attribute: _ }) => {
                belongs_to::generate_join_method(attr, input, &mut with_methods);
                belongs_to::append_result_collection(attr, &mut collections);
                belongs_to::build_row_processor(attr, &mut row_processors);
                belongs_to::build_entity_append(attr, &mut entity_appends);
            }
            Some(RelType::HasOneThrough { attribute: _ }) => {
                has_one_through::generate_join_method(attr, input, &mut with_methods);
                has_one_through::append_result_collection(attr, &mut collections);
                has_one_through::build_row_processor(attr, &mut row_processors);
                has_one_through::build_entity_append(attr, &mut entity_appends);
            }
            Some(RelType::HasManyThrough { attribute: _ }) => {
                has_many_through::generate_join_method(attr, input, &mut with_methods);
                has_many_through::append_result_collection(attr, &mut collections);
                has_many_through::build_row_processor(attr, &mut row_processors);
                has_many_through::build_entity_append(attr, &mut entity_appends);
            }
            _ => (),
        }
    }

    let with_methods_vec = with_methods.into_values().collect::<Vec<TokenStream>>();
    let collections_vec = collections.into_values().collect::<Vec<TokenStream>>();
    let row_processors_vec = row_processors.into_values().collect::<Vec<TokenStream>>();
    let entity_appends_vec = entity_appends.into_values().collect::<Vec<TokenStream>>();

    quote! {
        #[derive(Debug, Clone)]
        pub struct #repo_name {
            builder: ::dirtybase_contract::db_contract::base::query::QueryBuilder,
            manager: ::dirtybase_contract::db_contract::base::manager::Manager,
            eager: Vec<String>
        }


        impl #repo_name {
            pub fn new(manager: &::dirtybase_contract::db_contract::base::manager::Manager) -> Self {
                 let mut instance = Self {
                    builder:  ::dirtybase_contract::db_contract::base::query::QueryBuilder::new(
                        <#ident as ::dirtybase_contract::db_contract::table_entity::TableEntityTrait>::table_name(),
                        ::dirtybase_contract::db_contract::base::query::QueryAction::Query {columns: None}
                    ),
                    manager: manager.clone(),
                    eager: Vec::new()
                 };

                 instance
                    .builder
                    .select_multiple(&<#ident as ::dirtybase_contract::db_contract::table_entity::TableEntityTrait>::table_query_col_aliases(None));

                 instance
            }

            #(#with_methods_vec)*

            pub async fn get(&mut self) -> Result<Option<Vec<#ident>>, ::anyhow::Error> {
                let mut rows_map = ::std::collections::HashMap::new();
                #(#collections_vec)*

                let result = self.manager.execute_query(self.builder.clone()).all().await;

                *self = Self::new(&self.manager);

                match result {
                    Ok(Some(list)) => {
                        for row in &list {
                            if let Some(row_entity) = #ident::from_struct_column_value(row,
                                Some(<#ident as ::dirtybase_contract::db_contract::table_entity::TableEntityTrait>::table_name())) {
                                let row_hash = ::dirtybase_contract::db_contract::table_entity::TableEntityTrait::entity_hash(&row_entity);
                                rows_map.insert(row_hash, row_entity);

                                //joins
                                #(#row_processors_vec)*
                            }
                        }

                        // now map relationships
                        for(_, row_entity) in &mut rows_map {
                            let row_hash = ::dirtybase_contract::db_contract::table_entity::TableEntityTrait::entity_hash(row_entity);
                            #(#entity_appends_vec)*
                        }

                        Ok(Some(rows_map.into_iter().map(|e| e.1).collect::<Vec<#ident>>()))
                    },
                    Ok(None) => Ok(None),
                    Err(e) => Err(e),
                }
            }
        }
    }
}
