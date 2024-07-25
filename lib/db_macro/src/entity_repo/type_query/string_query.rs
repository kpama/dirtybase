use std::collections::HashMap;

use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::attribute_type::DirtybaseAttributes;

pub(crate) fn generate(
    columns: &HashMap<String, DirtybaseAttributes>,
    mut methods: Vec<TokenStream>,
    base_name: &Ident,
) -> Vec<TokenStream> {
    for (name, attr) in columns {
           if attr.the_type.as_str()  == "String" {
                let lower_name = name.to_lowercase();
                let by_method = format_ident!("{}", &lower_name);
                let like_method = format_ident!("{}_like", &lower_name);
                let all_like_method = format_ident!("all_{}_like", &lower_name);
                let start_with_method = format_ident!("{}_starts_with", &lower_name);
                let all_start_with_method = format_ident!("all_{}_start_with", &lower_name);
                let end_with_method = format_ident!("{}_ends_with", &lower_name);
                let all_end_with_method = format_ident!("all_{}_end_with", &lower_name);
                let contains_method = format_ident!("{}_contains", &lower_name);
                let all_contain_method = format_ident!("all_{}_contain", &lower_name);

                methods.push(quote!{
                  // foo()
                pub async fn #by_method<V: Into<dirtybase_db::field_values::FieldValue>>(&self, value: V) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
                  self.query_by(#name, value).fetch_one_to().await
                }
                // foo_like()
                pub async fn #like_method<V: Into<dirtybase_db::field_values::FieldValue>>(&self, value: V) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
                  self.manager.select_from_table(&self.table,|query| {
                      query.select_all()
                      .like(#name, value);
                    }).fetch_one_to().await
                }
                // all_foo_like()
                pub async fn #all_like_method<V: Into<dirtybase_db::field_values::FieldValue>>(&self, value: V) -> Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
                  self.manager.select_from_table(&self.table,|query| {
                      query.select_all()
                      .like(#name, value);
                    }).fetch_all_to().await
                }
                // foo_starts_with()
                pub async fn #start_with_method<V: Into<dirtybase_db::field_values::FieldValue>>(&self, value: V) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
                  self.manager.select_from_table(&self.table,|query| {
                    let f: dirtybase_db::field_values::FieldValue = value.into(); 
                      query.select_all()
                      .like(#name, format!("{}%", f));
                    }).fetch_one_to().await
                }
                // foo_ends_with()
                pub async fn #end_with_method<V: Into<dirtybase_db::field_values::FieldValue>>(&self, value: V) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
                  self.manager.select_from_table(&self.table,|query| {
                    let f: dirtybase_db::field_values::FieldValue = value.into(); 
                      query.select_all()
                      .like(#name, format!("%{}", f));
                    }).fetch_one_to().await
                }
                // foo_contains()
                pub async fn #contains_method<V: Into<dirtybase_db::field_values::FieldValue>>(&self, value: V) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
                  self.manager.select_from_table(&self.table,|query| {
                    let f: dirtybase_db::field_values::FieldValue = value.into(); 
                      query.select_all()
                      .like(#name, format!("%{}%", f));
                    }).fetch_one_to().await
                }
                // all_foo_start_with()
                pub async fn #all_start_with_method<V: Into<dirtybase_db::field_values::FieldValue>>(&self, value: V) -> Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
                  self.manager.select_from_table(&self.table,|query| {
                      let f: dirtybase_db::field_values::FieldValue = value.into(); 
                      query.select_all()
                      .like(#name, format!("{}%", f));
                    }).fetch_all_to().await
                }
                // all_foo_end_with()
                pub async fn #all_end_with_method<V: Into<dirtybase_db::field_values::FieldValue>>(&self, value: V) -> Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
                  self.manager.select_from_table(&self.table,|query| {
                      let f: dirtybase_db::field_values::FieldValue = value.into(); 
                      query.select_all()
                      .like(#name, format!("%{}", f));
                    }).fetch_all_to().await
                }
                // all_foo_contain()
                pub async fn #all_contain_method<V: Into<dirtybase_db::field_values::FieldValue>>(&self, value: V) -> Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
                  self.manager.select_from_table(&self.table,|query| {
                      let f: dirtybase_db::field_values::FieldValue = value.into(); 
                      query.select_all()
                      .like(#name, format!("%{}%", f));
                    }).fetch_all_to().await
                }
              });
            }
    }

    methods
}
