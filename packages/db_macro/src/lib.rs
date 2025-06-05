use helpers::*;
use proc_macro::TokenStream;

use quote::quote;
use syn::{DeriveInput, parse_macro_input};

mod attribute_type;
mod helpers;

#[proc_macro_derive(DirtyTable, attributes(dirty, dirty_rel))]
pub fn derive_dirtybase_entity(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let name = input.ident.clone();
    let columns_attributes = pluck_columns(&input);
    let table_name = pluck_table_name(&input);
    let id_column_method = build_id_method(&input);
    let foreign_id_method = build_foreign_id_method(&input, &table_name);

    let column = pluck_names(&columns_attributes);

    let generics = input.generics.clone();
    let (_impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let from_cv_for_handlers = names_of_from_cv_handlers(&columns_attributes);
    let from_cvs = build_from_handlers(&columns_attributes);
    let into_field_values = build_into_handlers(&columns_attributes);
    let into_cv_for_calls = build_into_for_calls(&columns_attributes);
    let special_column_methods = build_special_column_methods(&columns_attributes);
    let column_name_methods = build_prop_column_names_getter(&columns_attributes);
    let defaults = spread_default(&columns_attributes, &input);

    let expanded = quote! {

      // From columnValue methods for each field
      impl #ty_generics #name  #ty_generics #where_clause {

        #(#from_cvs)*

        #(#into_field_values)*

        #(#column_name_methods)*

        pub fn from_struct_column_value(mut cv: &mut ::dirtybase_contract::db_contract::types::StructuredColumnAndValue, key: Option<&str>) -> Option<Self> {
          if let Some(name) = key {
              if let Some(values) = cv.get(name) {
                    Some(values.clone().into())
                } else {
                    None
                }
          } else {
            Some(::dirtybase_contract::db_contract::types::FromColumnAndValue::from_column_value(cv.clone().fields()))
          }
        }

        pub fn into_embeddable(&self) -> ::dirtybase_contract::db_contract::field_values::FieldValue {
          ::dirtybase_contract::db_contract::field_values::FieldValue::from(self)
        }

      }

      // TableEntityTrait
      impl #ty_generics ::dirtybase_contract::db_contract::TableEntityTrait for #name  #ty_generics #where_clause {

        #id_column_method

        #foreign_id_method

        #(#special_column_methods)*

        fn table_name() -> &'static str {
          #table_name
        }

        fn table_columns() -> &'static [&'static str] {
          &[
             #(#column),*
          ]
        }
      }

      // FromColumnAndValue for T
      impl #ty_generics ::dirtybase_contract::db_contract::types::FromColumnAndValue  for #name  #ty_generics #where_clause {
        fn from_column_value(cv: ::dirtybase_contract::db_contract::types::ColumnAndValue) -> Self {
            Self {
                #(#from_cv_for_handlers),*,
                #defaults
            }
        }
      }

      // ToColumnAndValue for T
      impl #ty_generics ::dirtybase_contract::db_contract::types::ToColumnAndValue for #name  #ty_generics #where_clause {
        fn to_column_value(&self) -> Result<::dirtybase_contract::db_contract::types::ColumnAndValue, ::dirtybase_contract::anyhow::Error> {
            Ok(::dirtybase_contract::db_contract::ColumnAndValueBuilder::new()
                #(.#into_cv_for_calls)*
                .build())
        }
      }

      // Impl From FieldValue
      impl #ty_generics From<::dirtybase_contract::db_contract::field_values::FieldValue> for #name  #ty_generics #where_clause {
          fn from(value: ::dirtybase_contract::db_contract::field_values::FieldValue) -> Self {
             let cv = ::dirtybase_contract::db_contract::types::ColumnAndValue::from(value);
             if cv.is_empty() {
                Self::default()
             } else {
              ::dirtybase_contract::db_contract::types::FromColumnAndValue::from_column_value(cv)
             }
          }
      }

      impl #ty_generics From<&::dirtybase_contract::db_contract::field_values::FieldValue> for #name  #ty_generics #where_clause {
          fn from(value: &::dirtybase_contract::db_contract::field_values::FieldValue) -> Self {
            value.clone().into()
          }
      }

      // Impl from &Self to FieldValue
      impl #ty_generics From<&#name> for ::dirtybase_contract::db_contract::field_values::FieldValue {
          fn from(value: &#name ) -> ::dirtybase_contract::db_contract::field_values::FieldValue {
              ::dirtybase_contract::db_contract::field_values::FieldValue::Object(::dirtybase_contract::db_contract::types::ToColumnAndValue::to_column_value(value).expect("could not convert to field object"))
          }
      }

      // Impl from Self to FieldValue
      impl #ty_generics From<#name> for ::dirtybase_contract::db_contract::field_values::FieldValue {
          fn from(value: #name ) -> ::dirtybase_contract::db_contract::field_values::FieldValue {
            ::dirtybase_contract::db_contract::field_values::FieldValue::Object(::dirtybase_contract::db_contract::types::ToColumnAndValue::to_column_value(&value).expect("could not convert to field object"))
          }
      }

      // TODO: Generate a function that can be used in a migration to create the entity table

    };

    TokenStream::from(expanded)
}
