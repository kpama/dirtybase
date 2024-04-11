use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::{quote};

pub(crate) fn generate_repo_basic_query(base_name: &Ident, id_column: &str) -> Vec<TokenStream> {
    let mut methods = Vec::new();

    //  // general: query by a field
    methods.push(quote!{
           fn query_by<C: ToString, V: Into<dirtybase_db::field_values::FieldValue>>(&self, name: C, value: V) -> dirtybase_db::base::schema::SchemaWrapper {
             self.manager.select_from_table(&self.table, move |query| {
                query.select_all().eq(name, value);
             })
           }
     });

    //  // general: query by multiple fields
    //  methods.push(quote!{
    //         fn query_by_multi<C: ToString, V: Into<dirtybase_db::field_values::FieldValue>>(&self, name: C, kv: HashMap<C,V>) -> dirtybase_db::base::schema::SchemaWrapper {
    //           self.manager.select_from_table(&self.table, move |query| {
    //              query.select_all();
    //               for (key, value) in kv.into_iter().enumerate() {
    //                  query.eq(key, value.into());
    //               }
    //           })
    //         }
    //   });

    //  // fetch all
    //  methods.push(quote! {
    //      pub async fn all(&self) -> Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
    //       self.manager.select_from_table(&self.table,|query| {
    //            query.select_all();
    //         }).fetch_all_to().await
    //      }
    //  });

    //  // fetch one by
    //  methods.push(
    //    quote!{
    //        pub async fn one_by<C: ToString, V: Into<dirtybase_db::field_values::FieldValue>>(&self, name: C, value: V) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
    //          self.query_by(name, value).fetch_one_to().await
    //        }
    //    }
    //  );
    //  // fetch all by
    //  methods.push(
    //    quote!{
    //        pub async fn all_by<C: ToString, V: Into<dirtybase_db::field_values::FieldValue>>(&self, name: C, value: V) -> Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
    //          self.query_by(name, value).fetch_all_to().await
    //        }
    //    }
    //  );

    //  // fetch one by multi
    //  methods.push(
    //     quote!{
    //         pub async fn one_by_multi<C: ToString, V: Into<dirtybase_db::field_values::FieldValue>>(&self, name: C, kv: HashMap<C,V>) -> Result<Option<#base_name>, dirtybase_db::anyhow::Error> {
    //           self.query_by_multi(name, kv).fetch_one_to().await
    //         }
    //     }
    //   );

    //  // fetch all by
    //  methods.push(quote!{
    //         pub async fn all_by_multi<C: ToString, V: Into<dirtybase_db::field_values::FieldValue>>(&self, name: C, kv: HashMap<C,V>) ->  Result<Option<Vec<#base_name>>, dirtybase_db::anyhow::Error> {
    //           self.query_by_multi(name, kv).fetch_all_to().await
    //         }
    //   });

    // TODO: pagination
    // TODO: Streaming

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
