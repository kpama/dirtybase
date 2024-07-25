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
        if attr.optional {
            let lower_name = name.to_lowercase();
            let null_method = format_ident!("{}_is_null", &lower_name);
            let all_null_method = format_ident!("all_{}_is_null", &lower_name);
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
    methods
}
