use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput};

#[proc_macro_derive(InsertableDerive, attributes(field))]
pub fn entity_derive(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);

    let name = &ast.ident;

    eprintln!("{:#?}", &ast);

    let expanded = quote! {
        impl dirtybase_db_internal::entity::Insertable for #name {
            fn all() -> Vec<std::string::String> {
                vec!["Hello".into(), "World".into()]
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
    // TokenStream::new()
}
