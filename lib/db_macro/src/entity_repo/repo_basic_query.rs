use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::attribute_type::DirtybaseAttributes;

pub(crate) fn generate(
    columns: &Vec<(String, DirtybaseAttributes)>,
    mut methods: Vec<TokenStream>,
    base_name: &Ident,
    id_column: &str,
) -> Vec<TokenStream> {
    //  // general: query by a field
    methods.push(quote!{
           fn query_by<C: ToString, V: Into<dirtybase_db::field_values::FieldValue>>(&self, name: C, value: V) -> dirtybase_db::base::schema::SchemaWrapper {
             self.manager.select_from_table(&self.table, move |query| {
                query.select_all().eq(name, value);
             })
           }
           pub async fn all(&self) ->  Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
            self.manager.select_from_table(&self.table, |q| {
               q.select_all();
            }).fetch_all_to().await
           }
     });

    for (name, _attr) in columns {
        let lower_name = name.to_lowercase();

        let is_method = format_ident!("{}_is", &lower_name);
        let all_is_method = format_ident!("all_{}_is", &lower_name);
        let is_not_method = format_ident!("{}_is_not", &lower_name);
        let all_is_not_method = format_ident!("all_{}_is_not", &lower_name);
        //   let is_gt_method = format_ident!("{}_is_gt", &lower_name);
        //   let all_is_gt_method = format_ident!("all_{}_is_gt", &lower_name);
        //   let is_lt_method = format_ident!("{}_is_not", &lower_name);
        //   let all_is_lt_method = format_ident!("all_{}_is_not", &lower_name);
        //   let is_between_between = format_ident!("{}_is_between", &lower_name);
        //   let all_is_between_between = format_ident!("all_{}_is_between", &lower_name);
        //   let is_in_method = format_ident!("{}_is_in", &lower_name);
        //   let all_in_method = format_ident!("all_{}_is_in", &lower_name);

        methods.push(quote! {
            // foo_is(...)
            pub async fn #is_method<V: Into<dirtybase_db::field_values::FieldValue>>(&self, value: V) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
               self.query_by(#name, value).fetch_one_to().await
            }

            // all_foo_is(....)
            pub async fn #all_is_method<V: Into<dirtybase_db::field_values::FieldValue>>(&self, value: V) -> Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
               self.query_by(#name, value).fetch_all_to().await
            }

            // foo_is_not(...)
            pub async fn #is_not_method<V: Into<dirtybase_db::field_values::FieldValue>>(&self, value: V) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
               self.query_by(#name, value).fetch_one_to().await
            }

            // all_foo_is_not(....)
            pub async fn #all_is_not_method<V: Into<dirtybase_db::field_values::FieldValue>>(&self, value: V) -> Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
               self.query_by(#name, value).fetch_all_to().await
            }

        });
    }

    // fetch by id / ids
    if !id_column.is_empty() {
        methods.push(quote!{
          pub async fn id<V: Into<dirtybase_db::field_values::FieldValue>>(&self, id: V) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
             self.query_by(#id_column, id).fetch_one_to().await
          }

          pub async fn ids<V: Into<dirtybase_db::field_values::FieldValue> + IntoIterator >(&self, ids: V) -> Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
           self.manager.select_from_table(&self.table,|query| {
                query.select_all()
                .is_in(#id_column, ids);
             }).fetch_all_to().await
          }
       })
    }

    methods
}
