use crate::{attribute_type::DirtybaseAttributes, helpers::pluck_id_column};
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::DeriveInput;

pub(crate) fn generate_query_builder_struct(
    colums: &Vec<(String, DirtybaseAttributes)>,
    base_name: &Ident,
    table_name: &str,
    input: &DeriveInput,
) -> proc_macro2::TokenStream {
    let struct_name = format_ident!("{}Repo", base_name);
    let the_table_name = table_name.to_string();
    // let mut find_by_a_field = Vec::new();
    let mut methods = Vec::new();
    let id_column = pluck_id_column(input);

    if !id_column.is_empty() {
        methods.push(quote!{
          pub async fn id<V: Into<dirtybase_db::field_values::FieldValue>>(&self, id: V) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
             self.one_by(#id_column, id).await
          }

          pub async fn ids<V: Into<dirtybase_db::field_values::FieldValue> + IntoIterator >(&self, ids: V) -> Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
           self.manager.select_from_table(&self.table,|query| {
                query.select_all()
                .is_in(#id_column, ids);
             }).fetch_all_to().await
          }
       })
    } else {
        methods.push(quote!{
          pub async fn id<V: Into<dirtybase_db::field_values::FieldValue>>(&self, id: V) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
            panic!("Entity does not have a primary key");
          }

          pub async fn ids<V: Into<dirtybase_db::field_values::FieldValue> + IntoIterator >(&self, ids: V) -> Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
            panic!("Entity does not have a primary key");
          }
        });
    }

    for (name, attr) in colums {
        // println!("field type: {:?}", &attr.the_type);
        match attr.the_type.as_str() {
            "String" => {
                let by_method = format_ident!("{}", name.to_lowercase());
                let like_method = format_ident!("{}_like", name.to_lowercase());
                let all_like_method = format_ident!("all_{}_like", name.to_lowercase());
                let null_method = format_ident!("{}_is_null", name.to_lowercase());
                let all_null_method = format_ident!("all_{}_is_null", name.to_lowercase());
                methods.push(quote!{
                pub async fn #by_method<V: Into<dirtybase_db::field_values::FieldValue>>(&self, value: V) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
                  self.one_by(#name, value).await
                }
                pub async fn #like_method<V: Into<dirtybase_db::field_values::FieldValue>>(&self, value: V) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
                  self.manager.select_from_table(&self.table,|query| {
                      query.select_all()
                      .like(#name, value);
                    }).fetch_one_to().await
                }
                pub async fn #all_like_method<V: Into<dirtybase_db::field_values::FieldValue>>(&self, value: V) -> Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
                  self.manager.select_from_table(&self.table,|query| {
                      query.select_all()
                      .like(#name, value);
                    }).fetch_all_to().await
                }
              });
                if attr.optional {
                    methods.push(quote!{
                      pub async fn #null_method(&self, is_null: bool) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
                        self.manager.select_from_table(&self.table,|query| {
                          query.select_all();
                          if is_null {
                            query.is_null(#name);
                          } else {
                           query.is_not_null(#name);
                          }
                        }).fetch_one_to().await
                    }

                      pub async fn #all_null_method(&self, is_null: bool) -> Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
                        self.manager.select_from_table(&self.table,|query| {
                          query.select_all();
                          if is_null {
                            query.is_null(#name);
                          } else {
                           query.is_not_null(#name);
                          }
                        }).fetch_all_to().await
                    }
                })
                }
            }
            _ => (),
        }
    }

    quote! {
      #[derive(Clone)]
      pub struct  #struct_name {
        manager: std::sync::Arc<dirtybase_db::base::manager::Manager>,
        table: String,
      }

      impl #struct_name {

        pub fn new(manager: dirtybase_db::base::manager::Manager) -> Self {
            Self {
              manager: std::sync::Arc::new(manager),
              table: #the_table_name.to_string()
             }
        }


        pub fn builder(&self)-> dirtybase_db::base::query::EntityQueryBuilder<#base_name>{
          self.manager.table_for(&self.table)
        }

        pub fn manager(&self) -> &dirtybase_db::base::manager::Manager {
            &self.manager
        }

         pub async fn insert(&self, entity: #base_name) {
            let result = self.manager.insert(&self.table, entity).await;
         }

         pub async fn update<V: ToString>(&self, entity: #base_name, id: V, column: Option<&str>) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
          let cv = <#base_name as ::dirtybase_db::types::IntoColumnAndValue>::into_column_value(entity);
          let the_id = id.to_string();
          let id_ref = &the_id;

          let column_name = if let Some(c) = column {
            c
          }  else {
            #id_column
          };

          self.manager.update(&self.table, cv, |query|{
                query.eq(column_name, id_ref);
            }).await;

           self.one_by(column_name, the_id).await
         }

        pub async fn delete<V: ToString>(&self, id: V, column: Option<&str>) {
          let the_id = id.to_string();
          let id_ref = &the_id;

          let column_name = if let Some(c) = column {
            c
          }  else {
            #id_column
          };

            self.manager.delete(&self.table, |query|{
                query.eq(column_name, id_ref);
            }).await;
         }

          pub async fn all(&self) -> Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
            self.builder().all().await
          }


          pub async fn one_by<C: ToString, V: Into<dirtybase_db::field_values::FieldValue>>(&self, name: C, value: V) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
            let mut builder = self.builder();
            builder.query().eq(name, value);
            builder.one().await
          }

          pub async fn all_by<C: ToString, V: Into<dirtybase_db::field_values::FieldValue>>(&self, name: C, value: V) ->  Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
            let mut builder = self.builder();
            builder.query().eq(name, value);
            builder.all().await
          }


          pub fn save(entity: #base_name) -> Option<#base_name> {
             Some(entity)
          }

          #(#methods)*
      }

      #[busybody::async_trait]
      impl busybody::Injectable for #struct_name {
        async fn inject(c: &busybody::ServiceContainer) -> Self {
             let pool_manager: dirtybase_db::ConnectionPoolManager = c.get_type().unwrap();
             Self::new(pool_manager.default_schema_manager().unwrap())
        }
      }
    }
}
