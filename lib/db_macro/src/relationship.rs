use std::collections::HashMap;

use proc_macro2::TokenStream;
use proc_macro2::TokenTree;

pub(crate) mod belongs_to;
pub(crate) mod has_many;

use crate::attribute_type::{DirtybaseAttributes, RelType};

pub(crate) fn process_relation_attribute(
    _field: &syn::Field,
    dirty_attribute: &mut DirtybaseAttributes,
    token_stream: TokenStream,
) -> bool {
    let mut attributes = HashMap::<String, String>::new();

    if let Some(TokenTree::Group(g)) = token_stream.into_iter().next() {
        let mut it = g.stream().into_iter();
        loop {
            let item = it.next();
            if item.is_none() {
                break;
            }

            if let Some(TokenTree::Ident(key)) = item {
                let name = key.to_string().replace('\"', "");
                _ = it.next();
                if let Some(value) = it.next() {
                    attributes.insert(name, value.to_string().replace('\"', ""));
                }
            }
        }
    }

    dirty_attribute.relation = RelType::new(attributes);

    true
}
