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
    list: &mut Vec<TokenStream>,
) {
    if let Some(RelType::MorphMany { attribute }) = &attr.relation {
        // method
        let name = &attr.name;
        let method_name_st = format!("with_{name}");
        let method_name = format_ident!("{}", &method_name_st);
        let when_method_name = format_ident!("{}_when", &name);
        let method_name_where = format_ident!("{}_where", &method_name_st);
        let trashed_method_name = format_ident!("with_trashed_{}", &name);
        let trashed_method_name_where = format_ident!("with_trashed_{}_where", &name);
        let with_only_trashed_method_name = format_ident!("with_trashed_only_{}", &name);
        let with_only_trashed_method_name_where =
            format_ident!("with_trashed_only_{}_where", &name);
        let parent = format_ident!("{}", &input.ident);
        let foreign_type = format_ident!("{}", attr.the_type);
        let morph_name = if let Some(field) = &attribute.morph_name {
            field
        } else {
            std::panic!("morph relation must have a name. {name}");
        };
        let empty_callback = quote! {
            |_: &mut ::dirtybase_common::db::repo_relation::Relation<#parent>| {
                // nothing to do
            }
        };

        let morph_type = if let Some(field) = &attribute.morph_type {
            field
        } else {
            std::panic!("morph relation must have a type value. {name}");
        };

        let foreign_key_name = format!("{}_id", &morph_name);
        let morph_type_name = format!("{}_type", &morph_name);
        let mut morph_method_name = format_ident!("{}", &morph_type_name);

        // parent col
        let mut parent_col =
            quote! {<#parent as ::dirtybase_common::db::table_model::TableModel>::id_column()};
        // foreign col
        let mut foreign_col = quote! { #foreign_key_name };
        let mut morph_type_col = quote! { #morph_type_name };

        // parent col
        if let Some(field) = &attribute.local_col {
            parent_col = quote! { #field };
        }

        // foreign col
        if let Some(field) = &attribute.foreign_col {
            foreign_col = quote! { #field };
        }

        if let Some(field) = &attribute.morph_type_col {
            morph_type_col = quote! { #field };
            morph_method_name = format_ident!("{}", &field);
        }

        let trash_condition = if attribute.no_soft_delete {
            quote! {}
        } else {
            quote! {
                relation.query_mut().is_null(
                        <#foreign_type as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                        <#foreign_type as ::dirtybase_common::db::table_model::TableModel>::deleted_at_column().as_ref().unwrap()
                    )
                );
            }
        };

        list.push(quote! {
            pub fn #when_method_name<F>(&mut self , mut callback: F) -> &mut Self
                where F: FnMut(&mut ::dirtybase_common::db::repo_relation::Relation<#parent>)
             {

            let query = <#foreign_type as ::dirtybase_common::db::table_model::TableModel>::make_query_builder();

            let mut relation = ::dirtybase_common::db::repo_relation::Relation::<#parent>::new(
                ::dirtybase_common::db::repo_relation::RelationType::MorphMany{ query},
                |
                    relation: ::dirtybase_common::db::repo_relation::Relation<#parent>,
                    rows: &Vec<#parent>,
                    join_values: &mut ::std::collections::HashMap<String,::std::collections::HashMap<u64,::dirtybase_common::db::field_values::FieldValue>>
                | {
                    let (mut query, _) = relation.rel_type().builders();

                    query.select_multiple(&<#foreign_type as ::dirtybase_common::db::table_model::TableModel>::table_query_col_aliases(None));
                    query.is_eq(#morph_type_col, #morph_type);

                    let parent_col_name = #parent_col.to_string();

                    if join_values.get(&parent_col_name).is_none() {
                        let mut values = ::std::collections::HashMap::new();
                        let prefix = <#parent as ::dirtybase_common::db::table_model::TableModel>::table_name();
                        for  a_row in rows {
                            if let Ok(cv) = ::dirtybase_common::db::types::ToColumnAndValue::to_column_value(a_row) {
                                if let Some(v) = cv.get(&parent_col_name).cloned() {
                                   let hash = ::dirtybase_common::db::table_model::TableModel::entity_hash(a_row);
                                    values.insert(hash.clone(), v);
                                }
                            }
                        }
                        join_values.insert(parent_col_name.clone(), values);
                    }

                    let mut values = join_values.get(&parent_col_name).cloned().unwrap().into_values().collect::<Vec<
                        ::dirtybase_common::db::field_values::FieldValue
                    >>();

                    values.dedup();

                    query.is_in(
                       <#foreign_type as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(#foreign_col),
                        values);
                    ::dirtybase_common::db::repo_relation::RelationProcessor::new(query, parent_col_name, <#foreign_type as ::dirtybase_common::db::table_model::TableModel>::table_name().to_string(), #foreign_col.to_string())
                }
            );

             callback(&mut relation);
             self.relation.insert(#name.to_string(), relation);
             self
            }
        });

        let call_callback = quote! {
            callback(relation);
        };

        let token = quote! {
            pub fn #method_name(&mut self,) -> &mut Self {
                self.#method_name_where(#empty_callback)
            }

            pub fn #method_name_where<F>(&mut self, mut callback: F) -> &mut Self
             where F: FnMut(&mut ::dirtybase_common::db::repo_relation::Relation<#parent>)
            {
                self.#when_method_name(|relation| {
                    #call_callback
                    #trash_condition
                })
             }
        };

        list.push(token);
        list.push(quote! {
            pub fn #morph_method_name() -> &'static str {
                #morph_type
            }
        });

        if !attribute.no_soft_delete {
            list.push(quote! {
                pub fn #trashed_method_name(&mut self,) -> &mut Self {
                    self.#trashed_method_name_where(#empty_callback)
                }

                pub fn #trashed_method_name_where<F>(&mut self, mut callback: F) -> &mut Self
                    where F: FnMut(&mut ::dirtybase_common::db::repo_relation::Relation<#parent>)
                {
                    self.#when_method_name(|relation| {
                        #call_callback
                    })
                }
            });

            list.push(quote! {
                    pub fn #with_only_trashed_method_name(&mut self,) -> &mut Self {
                        self.#with_only_trashed_method_name_where(#empty_callback)
                    }

                    pub fn #with_only_trashed_method_name_where<F>(&mut self, mut callback: F) -> &mut Self
                        where F: FnMut(&mut ::dirtybase_common::db::repo_relation::Relation<#parent>) {
                            self.#when_method_name(|relation| {
                                #call_callback
                                relation.query_mut().is_not_null(
                                    <#foreign_type as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                                    <#foreign_type as ::dirtybase_common::db::table_model::TableModel>::deleted_at_column().as_ref().unwrap()
                                    )
                                );
                            })
                    }

                }
            );
        }
    }
}

pub(crate) fn build_entity_append(attr: &DirtybaseAttributes, list: &mut Vec<TokenStream>) {
    let name = &attr.name;
    let foreign_type = format_ident!("{}", attr.the_type);
    let name_ident = format_ident!("{}", name);

    let transform = quote! {
        related_rows.into_iter().map(|row|{
            #foreign_type::from_struct_column_value(
                &row,
                Some(<#foreign_type as ::dirtybase_common::db::table_model::TableModel>::table_name())
            )
        }).flatten().collect::<Vec<#foreign_type>>()
    };

    let body = if attr.optional {
        quote! {
                row_entity.#name_ident = Some(#transform);
        }
    } else {
        quote! {
                row_entity.#name_ident = #transform;
        }
    };

    let token = quote! {
        if let Some(rows) = rows_rel_map.get_mut(#name) {
            if let Some(related_rows) = rows.remove(&row_hash)  {
                #body
            }
        }
    };

    list.push(token);
}
