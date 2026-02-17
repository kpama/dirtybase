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
    let mut relationship_methods = HashMap::<String, Vec<TokenStream>>::new();
    let mut append_methods = Vec::<TokenStream>::new();

    for attr in columns_attributes.values() {
        let mut methods = Vec::new();
        match &attr.relation {
            Some(RelType::HasOne { attribute: _ }) => {
                has_one::generate_join_method(attr, input, &mut methods);
                has_one::build_entity_append(attr, &mut append_methods);
            }
            Some(RelType::HasMany { attribute: _ }) => {
                has_many::generate_join_method(attr, input, &mut methods);
                has_many::build_entity_append(attr, &mut append_methods);
            }
            Some(RelType::BelongsTo { attribute: _ }) => {
                belongs_to::generate_join_method(attr, input, &mut methods);
                belongs_to::build_entity_append(attr, &mut append_methods);
            }
            Some(RelType::HasOneThrough { attribute: _ }) => {
                has_one_through::generate_join_method(attr, input, &mut methods);
                has_one_through::build_entity_append(attr, &mut append_methods);
            }
            Some(RelType::HasManyThrough { attribute: _ }) => {
                has_many_through::generate_join_method(attr, input, &mut methods);
                has_many_through::build_entity_append(attr, &mut append_methods);
            }
            Some(RelType::MorphOne { attribute: _ }) => {
                morph_one::generate_join_method(attr, input, &mut methods);
                morph_one::build_entity_append(attr, &mut append_methods);
            }
            Some(RelType::MorphMany { attribute: _ }) => {
                morph_many::generate_join_method(attr, input, &mut methods);
                morph_many::build_entity_append(attr, &mut append_methods);
            }
            _ => (),
        }
        relationship_methods.insert(attr.name.to_string(), methods);
    }

    let relationship_methods = relationship_methods
        .into_values()
        .flatten()
        .collect::<Vec<TokenStream>>();
    let mut append_trash_filter = quote! {};
    let mut with_trashed = quote! {};
    let mut trashed_only = quote! {};
    let soft_deletable = !tbl_attr.no_soft_delete;
    let created_at = format_ident!("{}", tbl_attr.created_at_col);
    let updated_at = format_ident!("{}", tbl_attr.updated_at_col);
    let deleted_at = format_ident!("{}", tbl_attr.deleted_at_col);
    let id_field = format_ident!("{}", tbl_attr.id_field);
    let id_field_attr = columns_attributes
        .get(&tbl_attr.id_field)
        .expect("could not get entity ID field");
    let id_type = format_ident!("{}", id_field_attr.the_type);

    let mut column_names: Vec<proc_macro2::TokenStream> = Vec::new();

    for item in columns_attributes {
        if item.1.relation.is_some() {
            continue;
        }

        if item.1.flatten {
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
            let id = record.#id_field.clone().expect("expected a 'Some' ID but found 'None'");
        }
    } else {
        quote! {
            let id = record.#id_field.clone();
        }
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
            self.destroy_by_id(id).await
        }

        pub async fn destroy_by_id(&mut self, id: #id_type) -> Result<(), ::dirtybase_common::anyhow::Error> {
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
            _ = self.delete_by_id(id).await?;

            Ok(record)
        }

        pub async fn delete_by_id(&mut self, id: #id_type ) -> Result<(), ::dirtybase_common::anyhow::Error> {
            _ = self.manager.delete_from_table::<#ident>(|qb|{
                qb.is_eq(
                    <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                    <#ident as ::dirtybase_common::db::table_model::TableModel>::id_column())
                    ,id);
            }).await?;
            Ok(())
        }
    };

    if soft_deletable {
        delete_method = quote! {
            pub async fn delete(&mut self, mut record: #ident) -> Result<#ident, ::dirtybase_common::anyhow::Error>{
                record.#deleted_at = Some(::dirtybase_common::dirtybase_helper::time::current_datetime());
                self.update(record).await
            }

            pub async fn delete_by_id(&mut self, id: #id_type ) -> Result<(), ::dirtybase_common::anyhow::Error> {
                if let Some(record) = self.by_id(id).await? {
                    _ = self.delete(record).await?;
                }
                Ok(())
            }
        };

        restore_method = quote! {
            pub async fn restore(&mut self, id: #id_type) -> Result<Option<#ident>, ::dirtybase_common::anyhow::Error> {
                let name = <#ident as ::dirtybase_common::db::table_model::TableModel>::deleted_at_column().as_ref().expect("could not get entity `deleted at` column").to_string();

                let mut cv = ::std::collections::HashMap::new();
                cv.insert(
                    <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(name),
                     ::dirtybase_common::db::field_values::FieldValue::Null
                    );

                _ = self.manager.update_table::<#ident>(cv, |qb|{
                    qb.is_eq(
                            <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                        <#ident as ::dirtybase_common::db::table_model::TableModel>::id_column()),
                        id.clone());
                }).await?;

                self.by_id(id).await
            }
        };

        append_trash_filter = quote! {
            let flag_soft = "_soft_delete".to_string();
            if !self.settings.contains(&flag_soft) {
                self.builder.is_null(
                    <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                        <#ident as ::dirtybase_common::db::table_model::TableModel>::deleted_at_column().as_ref().expect("deleted at column is require")
                        )
                );
            }
        };

        with_trashed = quote! {
            pub fn with_trashed(&mut self,)  -> &mut Self {
                if self.settings.contains(&"_trashed_only".to_string()) {
                    return self;
                }

                let flag_soft = "_soft_delete".to_string();
                if !self.settings.contains(&flag_soft) {
                    self.settings.push(flag_soft.clone());
                }
                self
            }
        };

        trashed_only = quote! {
            pub fn trashed_only(&mut self)  -> &mut Self {
                let flag_soft = "_soft_delete";
                if let Some(index) = self.settings.iter().position(|entry| entry == flag_soft) {
                    _= self.settings.remove(index);
                }

                self.settings.push("_trashed_only".to_string());
                self.builder.is_not_null(
                        <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                            <#ident as ::dirtybase_common::db::table_model::TableModel>::deleted_at_column().as_ref().expect("deleted at column is require")
                            )
                        );

                self
            }
        };
    }

    quote! {
        #[derive(Debug, Clone)]
        pub struct #repo_name {
            builder: ::dirtybase_common::db::base::query::QueryBuilder,
            manager: ::dirtybase_common::db::base::manager::Manager,
            settings: Vec<String>,
            relation: ::std::collections::HashMap<String, ::dirtybase_common::db::repo_relation::Relation>,
            eager: Vec<String>,
        }


        impl #repo_name {
            pub fn new(manager: &::dirtybase_common::db::base::manager::Manager) -> Self {
                Self {
                    builder:  ::dirtybase_common::db::base::query::QueryBuilder::new(
                        <#ident as ::dirtybase_common::db::table_model::TableModel>::table_name(),
                        ::dirtybase_common::db::base::query::QueryAction::query()
                    ),
                    manager: manager.clone(),
                    eager: Vec::new(),
                    relation: ::std::collections::HashMap::new(),
                    settings: Vec::new(),
                }
            }

            #with_trashed

            #trashed_only

            #(#relationship_methods)*

            pub async fn get(&mut self) -> Result<Option<Vec<#ident>>, ::dirtybase_common::anyhow::Error> {
                let mut rows_map = ::std::collections::HashMap::<u64, #ident>::new();
                // <name of a field whos value is used in a join, <entry hash, the field value>>
                let mut join_field_values = ::std::collections::HashMap::new();
                //<String, ::std::collections::HashMap<u64,::dirtybase_common::db::field_values::FieldValue>>,
                let mut rows_rel_map = ::std::collections::HashMap::new();
                let id_column = <#ident as ::dirtybase_common::db::table_model::TableModel>::id_column();
                let table = <#ident as ::dirtybase_common::db::table_model::TableModel>::table_name();
                #append_trash_filter


                self
                    .builder
                    .select_multiple(&<#ident as ::dirtybase_common::db::table_model::TableModel>::table_query_col_aliases(None));

                let result = self.manager.execute_query(self.builder.clone()).all().await;

                match result {
                    Ok(Some(mut raw_list)) => {
                        for row in &mut raw_list {

                            if let Some(row_entity) = #ident::from_struct_column_value(row,
                                Some(<#ident as ::dirtybase_common::db::table_model::TableModel>::table_name())) {
                                let row_hash= ::dirtybase_common::db::table_model::TableModel::entity_hash(&row_entity);
                                row.fields_mut().insert("__hash".to_string(), row_hash.into());
                                rows_map.insert(row_hash, row_entity);
                            }

                        }

                        for (name, rel) in &self.relation {
                           if let Err(e)  = rel.clone().process(name, &self.manager, &raw_list, &mut join_field_values, &mut rows_rel_map).await {
                                *self = Self::new(&self.manager);
                                return Err(e);
                           }
                        }

                        // now map relationships
                        for(row_hash, row_entity) in &mut rows_map {
                            #(#append_methods)*
                        }

                        *self = Self::new(&self.manager);
                        Ok(Some(rows_map.into_iter().map(|e| e.1).collect::<Vec<#ident>>()))
                    },
                    Ok(None) => {
                        *self = Self::new(&self.manager);
                        Ok(None)
                    },
                    Err(e) => {
                        *self = Self::new(&self.manager);
                        Err(e)
                    },
                }
            }

            pub async fn one(&mut self) -> Result<Option<#ident>, ::dirtybase_common::anyhow::Error> {
                match self.limit(1).get().await {
                    Ok(Some(mut list)) => {
                        Ok(list.pop())
                    },
                    Err(e) => Err(e),
                    _ => Ok(None)
                }
            }

            pub fn limit(&mut self, limit: usize) -> &mut Self {
                 self.builder.limit(limit);
                 self
            }

            pub async fn latest(&mut self)-> Result<Option<#ident>, ::dirtybase_common::anyhow::Error>  {
                self.builder.desc(
                  <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                        <#ident as ::dirtybase_common::db::table_model::TableModel>::id_column())
                );

                self.one().await
            }

            pub async fn oldest(&mut self)-> Result<Option<#ident>, ::dirtybase_common::anyhow::Error>  {
                self.builder.asc(
                  <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                        <#ident as ::dirtybase_common::db::table_model::TableModel>::id_column())
                );

                self.one().await
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
                        Err(result.err().expect("could not run 'count' query"))
                  }

            }

            pub fn filter(&mut self, mut callback: impl FnOnce(&mut ::dirtybase_common::db::base::query::QueryBuilder)) -> &mut Self {
                callback(&mut self.builder);
                self
            }

            pub async fn by_id(&mut self, id: #id_type) -> Result<Option<#ident>, ::dirtybase_common::anyhow::Error> {
                self.builder.is_eq(
                  <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                        <#ident as ::dirtybase_common::db::table_model::TableModel>::id_column()),
                    id);
                self.one().await
            }

            pub async fn id_in(&mut self, ids: Vec<#id_type>)-> Result<Option<Vec<#ident>>, ::dirtybase_common::anyhow::Error> {
                self.builder.is_in(
                        <#ident as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                        <#ident as ::dirtybase_common::db::table_model::TableModel>::id_column()),
                        ids
                );
                self.get().await
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
