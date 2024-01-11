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
    let the_table_name = format!("{}", table_name);
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

          pub async fn all(&self) -> Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
           self.manager.select_from_table(&self.table,|query| {
                query.select_all();
             }).fetch_all_to().await
          }


          pub async fn one_by<C: ToString, V: Into<dirtybase_db::field_values::FieldValue>>(&self, name: C, value: V) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
            self.manager.select_from_table(&self.table, move |query| {
               query.select_all()
                  .eq(name, value);
            }).fetch_one_to().await
          }

          pub async fn all_by<C: ToString, V: Into<dirtybase_db::field_values::FieldValue>>(&self, name: C, value: V) ->  Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
           self.manager.select_from_table(&self.table, move |query| {
                query.select_all()
                   .eq(name, value);
             }).fetch_all_to().await
          }


          pub fn save(entity: #base_name) -> Option<#base_name> {
             Some(entity)
          }

          #(#methods)*
      }
    }
}
