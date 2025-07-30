use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

use crate::attribute_type::RelType;
use crate::attribute_type::{DirtybaseAttributes, RelationAttribute};

pub(crate) fn build_attribute(
    attr: HashMap<String, String>,
    _field: &syn::Field,
    _input: &DeriveInput,
) -> RelationAttribute {
    //
    RelationAttribute::from(attr)
}

pub(crate) fn generate_join_method(
    attr: &DirtybaseAttributes,
    input: &DeriveInput,
    list: &mut HashMap<String, TokenStream>,
) {
    if let Some(RelType::HasOne { attribute }) = &attr.relation {
        // method
        let name = &attr.name;
        let method_name_st = format!("with_{name}");
        let method_name = format_ident!("{}", &method_name_st);
        let trashed_method_name = format_ident!("with_trashed_{}", &name);
        let with_only_trashed_method_name = format_ident!("with_trashed_only_{}", &name);
        let parent = format_ident!("{}", &input.ident);
        let foreign_type = format_ident!("{}", attr.the_type);

        // parent col
        let mut parent_col = quote! {<#parent as ::dirtybase_contract::db_contract::table_model::TableModel>::id_column()};
        // foreign col
        let mut foreign_col = quote! { <#parent as ::dirtybase_contract::db_contract::table_model::TableModel>::foreign_id_column() };

        // parent col
        if let Some(field) = &attribute.local_col {
            parent_col = quote! { #field };
        }

        // foreign key
        if let Some(field) = &attribute.foreign_col {
            foreign_col = quote! { #field };
        }

        let trash_condition = if attribute.no_soft_delete {
            quote! {}
        } else {
            quote! {
                 self.builder.is_null(
                    <#foreign_type as ::dirtybase_contract::db_contract::table_model::TableModel>::prefix_with_tbl(
                        <#foreign_type as ::dirtybase_contract::db_contract::table_model::TableModel>::deleted_at_column().as_ref().unwrap()
                    )
                );
            }
        };

        let token = quote! {
            pub fn #method_name(&mut self,) -> &mut Self {
                let name = #name.to_string();
                if !self.eager.contains(&name) {
                    #trash_condition
                    self.builder.inner_join_table_and_select::<#parent, #foreign_type>(#parent_col, #foreign_col, None);
                    self.eager.push(name);
                }
                self
            }
        };

        list.insert(method_name_st, token);

        if !attribute.no_soft_delete {
            list.insert("rel_with_trashed".to_string(),
                quote! {
                    pub fn #trashed_method_name(&mut self,) -> &mut Self {
                        let name = #name.to_string();
                        if !self.eager.contains(&name) {
                            self.builder.inner_join_table_and_select::<#parent, #foreign_type>(#parent_col, #foreign_col, None);
                            self.eager.push(name);
                        }
                        self
                    }
                }
            );

            list.insert("with_only_trashed_method".to_string(), 
                quote! {
                    pub fn #with_only_trashed_method_name(&mut self,) -> &mut Self {
                        let name = #name.to_string();
                        if !self.eager.contains(&name) {
                            self.builder.is_not_null(
                                <#foreign_type as ::dirtybase_contract::db_contract::table_model::TableModel>::prefix_with_tbl(
                                    <#foreign_type as ::dirtybase_contract::db_contract::table_model::TableModel>::deleted_at_column().as_ref().unwrap()
                                )
                            );
                            self.builder.inner_join_table_and_select::<#parent, #foreign_type>(#parent_col, #foreign_col, None);
                            self.eager.push(name);
                        }
                        self
                    }
                }
            );
        }
    }
}

pub(crate) fn append_result_collection(
    attr: &DirtybaseAttributes,
    list: &mut HashMap<String, TokenStream>,
) {
    let name = &attr.name;
    let foreign_type = format_ident!("{}", attr.the_type);
    let map_name_st = format!("{name}_map");
    let map_name = format_ident!("{}", &map_name_st);
    let is_eager = format_ident!("are_{}_eager", name);
    let token = quote! {
        let mut #map_name: ::std::collections::HashMap::<u64,#foreign_type> = ::std::collections::HashMap::new();
        let #is_eager = self.eager.contains(&#name.to_string());
    };
    list.insert(map_name_st, token);
}

pub(crate) fn build_row_processor(
    attr: &DirtybaseAttributes,
    list: &mut HashMap<String, TokenStream>,
) {
    let name = &attr.name;
    let is_eager = format_ident!("are_{}_eager", name);
    let map_name_st = format!("{name}_map");
    let map_name = format_ident!("{}", &map_name_st);
    let foreign_type = format_ident!("{}", attr.the_type);

    let token = quote! {
       //
       if #is_eager {
            if let Some(entity) = #foreign_type::from_struct_column_value(row,
                 Some(<#foreign_type as ::dirtybase_contract::db_contract::table_model::TableModel>::table_name())) {
                #map_name.insert(row_hash ,entity);
            }
       }
    };

    list.insert(map_name_st, token);
}

pub(crate) fn build_entity_append(
    attr: &DirtybaseAttributes,
    list: &mut HashMap<String, TokenStream>,
) {
    let name = &attr.name;
    let is_eager = format_ident!("are_{}_eager", name);
    let map_name_st = format!("{name}_map");
    let map_name = format_ident!("{}", &map_name_st);
    let name_ident = format_ident!("{}", name);

    let body = if attr.optional {
        quote! {
                row_entity.#name_ident = Some(parent);
        }
    } else {
        quote! {
                row_entity.#name_ident = parent;
        }
    };

    let token = quote! {
        //
        if #is_eager {
            //
            if let Some(parent) = #map_name.remove(&row_hash) {
                #body
            }
        }
    };

    list.insert(map_name_st, token);
}
