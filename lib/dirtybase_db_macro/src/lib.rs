use attribute_type::AttributeType;
use proc_macro::{TokenStream, TokenTree};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Meta, MetaList};

mod attribute_type;

#[proc_macro_derive(DirtyTable, attributes(dirty))]
pub fn derive_dirtybase_entity(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    dbg!(&input);

    let table_name = pluck_table_name(&input);
    let column = pluck_columns(&input);
    let column2 = pluck_columns(&input);
    let name = input.ident;
    let generics = input.generics;

    let struct_fields: Vec<syn::Ident> = column2.iter().map(|x| format_ident!("{}", x)).collect();

    let ff = quote! {
        Self {
        #(#struct_fields : dirtybase_db_types::field_values::FieldValue::from_ref_option_into(cv.get(#column2))),*,
         ..Self::default()
        }
    };

    let expanded = quote! {
      impl #generics dirtybase_db_types::TableEntityTrait for #generics #name {
        fn table_name() -> &'static str {
          #table_name
        }

        fn table_columns() -> &'static [&'static str] {
          &[
             #(#column),*
          ]
        }
      }

      impl #generics dirtybase_db_types::types::FromColumnAndValue  for #generics #name {

        fn from_column_value(cv: dirtybase_db_types::types::ColumnAndValue) -> Self {
            // #(let a1 =   dirtybase_db_types::types::FieldValue::from_ref_option_into(cv.get(#column)));*
            // Self {
                //  #(format_ident!({}:{}, #column2, dirtybase_db_types::types::FieldValue::from_ref_option_into(cv.get(#column2)))),*
                // ..Self::default()
            // }
            #ff
        }
      }

    };

    TokenStream::from(expanded)
}

fn pluck_table_name(input: &DeriveInput) -> String {
    let mut table_name = "".to_owned();

    for attr in &input.attrs {
        match &attr.meta {
            Meta::List(the_list) => {
                if the_list.path.is_ident("dirty") {
                    let mut walker = the_list.tokens.clone().into_iter();
                    if let Some(arg) = walker.next() {
                        if arg.to_string() == "table" {
                            if let Some(tbl) = walker.nth(1) {
                                table_name = tbl.to_string().replace("\"", "");
                            }
                        }
                    }
                    break;
                }
            }
            _ => (),
        }
    }

    table_name
}

fn pluck_columns(input: &DeriveInput) -> Vec<String> {
    let mut column = Vec::new();

    match &input.data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => {
                // dbg!(&fields.named);
                for a_field in fields.named.iter() {
                    if let Some(a_col) = get_real_column_name(a_field) {
                        column.push(a_col);
                    }
                    // dbg!(a_field);
                }
            }
            _ => (),
        },
        _ => (),
    }
    // dbg!(input);

    column
}

fn get_real_column_name(field: &syn::Field) -> Option<String> {
    let mut name = "".to_string();
    // = field.ident.as_ref().unwrap();
    if field.attrs.len() > 0 {
        for attr in &field.attrs {
            // dbg!(&attr.meta);
            match &attr.meta {
                Meta::List(the_list) => {
                    if the_list.path.is_ident("dirty") {
                        let mut walker = the_list.tokens.clone().into_iter();
                        if let Some(key) = walker.next() {
                            match key.to_string().as_str() {
                                "col" => {
                                    name = walker.nth(1).unwrap().to_string().replace("\"", "");
                                }
                                "," => {}
                                "from" | "into" => {
                                    if name.is_empty() {
                                        name = field.ident.as_ref().unwrap().to_string();
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    } else {
        name = field.ident.as_ref().unwrap().to_string();
    }

    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn field_attributes(metalist: &MetaList) -> Vec<AttributeType> {
    let mut walker = metalist.tokens.clone().into_iter();
    let mut attr = Vec::new();

    while walker.count() > 0 {
        if let (t, wa) = attribute_to_attribute_type(walker) {
            walker = wa;
            if t != AttributeType::Unknown {
                attr.push(t);
            }
        }
    }

    attr
}

fn attribute_to_attribute_type(mut walker: IntoIter) -> (AttributeType, IntoIter) {
    if let Some(key) = walker.next() {
        return match key.to_string().as_str() {
            "col" => (
                AttributeType::ColName(walker.nth(1).unwrap().to_string().replace("\"", "").into()),
                walker,
            ),
            "," => attribute_to_attribute_type(walker),
            "from" => (
                AttributeType::FromHandler(
                    walker.nth(1).unwrap().to_string().replace("\"", "").into(),
                ),
                walker,
            ),
            "into" => (
                AttributeType::IntoHandler(
                    walker.nth(1).unwrap().to_string().replace("\"", "").into(),
                ),
                walker,
            ),
            _ => (AttributeType::Unknown, walker),
        };
    }

    (AttributeType::Unknown, walker)
}
