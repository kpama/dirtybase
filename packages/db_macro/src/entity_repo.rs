use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

use crate::{
    attribute_type::{DirtybaseAttributes, RelType, TableAttribute},
    relationship::{
        belongs_to, has_many, has_many_through, has_one, has_one_through, morph_many, morph_one,
    },
};

pub fn build_entity_repo(
    input: &DeriveInput,
    columns_attributes: &HashMap<String, DirtybaseAttributes>,
    tbl_attr: &TableAttribute,
) -> TokenStream {
    //
    let ident = input.ident.clone();
    let repo_name = format_ident!("{}Repo", &input.ident);
    let mut with_methods = HashMap::<String, TokenStream>::new();
    let mut collections = HashMap::<String, TokenStream>::new();
    let mut row_processors = HashMap::<String, TokenStream>::new();
    let mut entity_appends = HashMap::<String, TokenStream>::new();

    for attr in columns_attributes.values() {
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
            Some(RelType::MorphOne { attribute: _ }) => {
                morph_one::generate_join_method(attr, input, &mut with_methods);
                morph_one::append_result_collection(attr, &mut collections);
                morph_one::build_row_processor(attr, &mut row_processors);
                morph_one::build_entity_append(attr, &mut entity_appends);
            }
            Some(RelType::MorphMany { attribute: _ }) => {
                morph_many::generate_join_method(attr, input, &mut with_methods);
                morph_many::append_result_collection(attr, &mut collections);
                morph_many::build_row_processor(attr, &mut row_processors);
                morph_many::build_entity_append(attr, &mut entity_appends);
            }
            _ => (),
        }
    }

    let with_methods_vec = with_methods.into_values().collect::<Vec<TokenStream>>();
    let collections_vec = collections.into_values().collect::<Vec<TokenStream>>();
    let row_processors_vec = row_processors.into_values().collect::<Vec<TokenStream>>();
    let entity_appends_vec = entity_appends.into_values().collect::<Vec<TokenStream>>();
    let mut append_trash_filter = quote! {};
    let mut with_trashed = quote! {};
    let mut trashed_only = quote! {};
    let soft_deletable = !tbl_attr.no_soft_delete;
    let created_at = format_ident!("{}", tbl_attr.created_at_col);
    let updated_at = format_ident!("{}", tbl_attr.updated_at_col);
    let deleted_at = format_ident!("{}", tbl_attr.deleted_at_col);
    let id_field = format_ident!("{}", tbl_attr.id_field);
    let id_field_attr = columns_attributes.get(&tbl_attr.id_field).unwrap();
    let id_type = format_ident!("{}", id_field_attr.the_type);

    let mut column_names: Vec<proc_macro2::TokenStream> = Vec::new();

    for item in columns_attributes {
        if item.1.relation.is_some() {
            continue;
        }

        let fn_name = format_ident!("col_{}", item.0);
        let col_name = &item.1.name;
        let full_name = format!("{}.{}", &tbl_attr.table_name, col_name);
        column_names.push(quote! {
                pub fn #fn_name() -> &'static str {
                     #full_name
                }
        })
    }

    // Makes a copy of the current record ID value
    let pluck_rec_id = if id_field_attr.optional {
        quote! {
            let id = record.#id_field.clone().unwrap();
        }
    } else {
        quote! {
            let id = record.#id_field.clone();
        }
    };

    let instance = quote! {
        let mut instance = Self {
            builder:  ::dirtybase_common::db::base::query::QueryBuilder::new(
                <#ident as ::dirtybase_common::db::table_model::TableModel>::table_name(),
                        ::dirtybase_common::db::base::query::QueryAction::Query {columns: None}
                    ),
            manager: manager.clone(),
            eager: Vec::new(),
        };
    };

    // insert
    let set_created_at = if tbl_attr.no_timestamp {
        quote! {}
    } else {
        quote! {
            record.#created_at = Some(::dirtybase_common::dirtybase_helper::time::current_datetime());
        }
    };
    let insert_method = quote! {
        pub async fn insert(&mut self, mut record: #ident) -> Result<#ident, ::dirtybase_common::anyhow::Error> {
            #set_created_at
            #pluck_rec_id

           _ = self.manager.insert_into::<#ident>(record).await?;

            match self.by_id(id).await? {
                Some(v) => Ok(v),
                None => Err(::dirtybase_common::anyhow::anyhow!("could not retrieve inserted model"))
            }
        }
    };

    // update
    let set_updated_at = if tbl_attr.no_timestamp {
        quote! {}
    } else {
        quote! {
            record.#updated_at= Some(::dirtybase_common::dirtybase_helper::time::current_datetime());
        }
    };
    let update_method = quote! {
        pub async fn update(&mut self, mut record: #ident) -> Result<#ident, ::dirtybase_common::anyhow::Error>{
            #set_updated_at
            #pluck_rec_id

            _ = self.manager.update_table::<#ident>(record, |qb| {
                qb.is_eq(
                    <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                    <#ident as ::dirtybase_common::db::table_model::TableModel>::id_column()
                    ), id.clone());
            }).await?;

            match self.by_id(id).await? {
                Some(v) =>Ok(v),
                None => Err(::dirtybase_common::anyhow::anyhow!("could not retrieve updated model"))
            }
        }
    };

    let mut restore_method = quote! {};

    // destroy record
    let destroy_method = quote! {
        pub async fn destroy(&mut self, record: #ident) -> Result<(), ::dirtybase_common::anyhow::Error> {
            #pluck_rec_id

            self.manager.delete_from_table::<#ident>(|qb|{
                qb.is_eq(
                    <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                    <#ident as ::dirtybase_common::db::table_model::TableModel>::id_column())
                    , id);
            }).await
        }
    };

    let mut delete_method = quote! {
        pub async fn delete(&mut self, mut record: #ident) -> Result<#ident, ::dirtybase_common::anyhow::Error> {
            #pluck_rec_id

            _ = self.manager.delete_from_table::<#ident>(|qb|{
                qb.is_eq(

                    <#ident as ::dirtybase_common::db::table_model::TableModel>::id_column(),  id);
            }).await?;

            Ok(record)
        }
    };

    if soft_deletable {
        delete_method = quote! {
            pub async fn delete(&mut self, mut record: #ident) -> Result<#ident, ::dirtybase_common::anyhow::Error>{
                record.#deleted_at = Some(::dirtybase_common::dirtybase_helper::time::current_datetime());
                self.update(record).await
            }
        };

        restore_method = quote! {
            pub async fn restore(&mut self, id: #id_type) -> Result<Option<#ident>, ::dirtybase_common::anyhow::Error> {
                let name = <#ident as ::dirtybase_common::db::table_model::TableModel>::deleted_at_column().as_ref().unwrap().to_string();

                let mut cv = ::std::collections::HashMap::new();
                cv.insert(name, ::dirtybase_common::db::field_values::FieldValue::Null);

                _ = self.manager.update_table::<#ident>(cv, |qb|{
                    qb.is_eq(<#ident as ::dirtybase_common::db::table_model::TableModel>::id_column(), id.clone());
                }).await?;

                self.by_id(id).await
            }
        };

        append_trash_filter = quote! {
            if !self.eager.contains(&"_soft_delete".to_string()) {
                self.builder.is_null(
                    <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                        <#ident as ::dirtybase_common::db::table_model::TableModel>::deleted_at_column().as_ref().unwrap()
                        )
                );
            }
        };

        with_trashed = quote! {
            pub fn with_trashed(&mut self,)  -> &mut Self {
                let flag_soft = "_soft_delete".to_string();
                if !self.eager.contains(&flag_soft) {
                    self.eager.push(flag_soft);
                }
                self
            }
        };

        trashed_only = quote! {
            pub fn trashed_only(&mut self)  -> &mut Self {
                let flag_soft = "_soft_delete".to_string();
                if !self.eager.contains(&flag_soft) {
                    self.builder.is_not_null(
                        <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                            <#ident as ::dirtybase_common::db::table_model::TableModel>::deleted_at_column().as_ref().unwrap()
                            )
                        );
                    self.eager.push(flag_soft);
                 }

                self
            }
        };
    }

    quote! {
        #[derive(Debug, Clone)]
        pub struct #repo_name {
            builder: ::dirtybase_common::db::base::query::QueryBuilder,
            manager: ::dirtybase_common::db::base::manager::Manager,
            eager: Vec<String>,
        }


        impl #repo_name {
            pub fn new(manager: &::dirtybase_common::db::base::manager::Manager) -> Self {
                #instance
                instance
            }

            #with_trashed

            #trashed_only

            #(#with_methods_vec)*

            pub async fn get(&mut self) -> Result<Option<Vec<#ident>>, ::dirtybase_common::anyhow::Error> {
                let mut rows_map = ::std::collections::HashMap::new();
                #(#collections_vec)*
                #append_trash_filter

                self
                    .builder
                    .select_multiple(&<#ident as ::dirtybase_common::db::table_model::TableModel>::table_query_col_aliases(None));

                let result = self.manager.execute_query(self.builder.clone()).all().await;

                *self = Self::new(&self.manager);

                match result {
                    Ok(Some(list)) => {
                        for row in &list {
                            if let Some(row_entity) = #ident::from_struct_column_value(row,
                                Some(<#ident as ::dirtybase_common::db::table_model::TableModel>::table_name())) {
                                let row_hash = ::dirtybase_common::db::table_model::TableModel::entity_hash(&row_entity);
                                rows_map.insert(row_hash, row_entity);

                                //joins
                                #(#row_processors_vec)*
                            }
                        }

                        // now map relationships
                        for(row_hash, row_entity) in &mut rows_map {
                            #(#entity_appends_vec)*
                        }

                        Ok(Some(rows_map.into_iter().map(|e| e.1).collect::<Vec<#ident>>()))
                    },
                    Ok(None) => Ok(None),
                    Err(e) => Err(e),
                }
            }

            pub async fn first(&mut self) -> Result<Option<#ident>, ::dirtybase_common::anyhow::Error> {
                match self.get().await {
                    Ok(Some(mut list)) => {
                        Ok(list.pop())
                    },
                    Err(e) => Err(e),
                    _ => Ok(None)
                }
            }

            pub async fn latest(&mut self)-> Result<Option<#ident>, ::dirtybase_common::anyhow::Error>  {
                self.builder.desc(
                  <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                        <#ident as ::dirtybase_common::db::table_model::TableModel>::id_column())
                );

                self.first().await
            }

            pub async fn oldest(&mut self)-> Result<Option<#ident>, ::dirtybase_common::anyhow::Error>  {
                self.builder.asc(
                  <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                        <#ident as ::dirtybase_common::db::table_model::TableModel>::id_column())
                );

                self.first().await
            }

            pub async fn count(&mut self)-> Result<i64, ::dirtybase_common::anyhow::Error> {
                #append_trash_filter

                let id_column = <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                        <#ident as ::dirtybase_common::db::table_model::TableModel>::id_column());

                self.builder.count_as(
                    &id_column,
                    "_count_all"
                );

                let result = self.manager.execute_query(self.builder.clone()).fetch_one().await;
                *self = Self::new(&self.manager);

                if let Ok(row) = result {
                    match row {
                        Some(r) => {
                           let count = if let Some(v) = r.get("_count_all") {
                                ::std::primitive::i64::from(v)
                            } else {
                                0
                            };
                           Ok(count)
                        }
                        None => Ok(0),
                    }
                  } else {
                        Err(result.err().unwrap())
                  }

            }

            pub async fn by_id(&mut self, id: #id_type) -> Result<Option<#ident>, ::dirtybase_common::anyhow::Error> {
                self.builder.is_eq(
                  <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                        <#ident as ::dirtybase_common::db::table_model::TableModel>::id_column()),
                    id);
                self.first().await
            }

            pub fn table_name() -> &'static str {
               <#ident as ::dirtybase_common::db::table_model::TableModel>::table_name()
            }

            #insert_method
            #update_method
            #delete_method
            #destroy_method
            #restore_method
            #(#column_names)*
        }
    }
}
