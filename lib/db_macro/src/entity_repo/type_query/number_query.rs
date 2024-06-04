use proc_macro2::Ident;
use proc_macro2::TokenStream;
// use quote::{format_ident, quote};

use crate::attribute_type::DirtybaseAttributes;

pub(crate) fn generate(
    columns: &Vec<(String, DirtybaseAttributes)>,
    methods: Vec<TokenStream>,
    _base_name: &Ident,
) -> Vec<TokenStream> {
    for (_name, attr) in columns {
        match attr.the_type.as_str() {
            "usize" | "isize" | "i128" | "u128" | "f64" | "f32" | "i64" | "u64" | "i32" | "u32"
            | "i16" | "u16" | "i8" | "u8" => {
                // TODO: implement this feature
            }
            _ => (),
        }
    }

    methods
}
