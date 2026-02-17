use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::DeriveInput;

use crate::attribute_type::{DirtybaseAttributes, RelationAttribute};
use crate::attribute_type::{RelType};

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
    if let Some(RelType::HasManyThrough { attribute }) = &attr.relation {
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

        let pivot_type = if let Some(p) = &attribute.pivot {
            format_ident!("{}", p)
        } else {
            std::panic!("pivot type not specified for: {name}");
        };

        // parent col
        let mut parent_col =
            quote! {<#parent as ::dirtybase_common::db::table_model::TableModel>::id_column()};
        // foreign col
        let mut foreign_col = quote! { <#parent as ::dirtybase_common::db::table_model::TableModel>::foreign_id_column() };
        // pivot foreign col
        let mut pivot_through_col = quote! { <#foreign_type as ::dirtybase_common::db::table_model::TableModel>::foreign_id_column() };
        let mut through_col = quote! { <#foreign_type as ::dirtybase_common::db::table_model::TableModel>::id_column() };
        let empty_callback = quote!{
            |_: &mut ::dirtybase_common::db::repo_relation::Relation| {
                // nothing to do
            }
        };

        // parent table column
        if let Some(field) = &attribute.local_col {
            parent_col = quote! { #field };
        }

        // foreign table column
        if let Some(field) = &attribute.foreign_col {
            foreign_col = quote! { #field };
        }

        // pivot table column
        if let Some(field) = &attribute.pivot_through_col {
            pivot_through_col = quote! { #field };
        }

        if let Some(field) = &attribute.through_col {
            through_col = quote! { #field };
        }

        let trash_condition = if attribute.no_soft_delete {
            quote! {}
        } else {
            quote! {
                relation.query_mut()
                 .is_null(
                    <#foreign_type as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                        <#foreign_type as ::dirtybase_common::db::table_model::TableModel>::deleted_at_column().as_ref().unwrap()
                    )
                 );
            }
        };

        list.push(quote! {
            pub fn #when_method_name<F>(&mut self , mut callback: F) -> &mut Self
                where F: FnMut(&mut ::dirtybase_common::db::repo_relation::Relation)
             {
                let query = <#foreign_type as ::dirtybase_common::db::table_model::TableModel>::make_query_builder();
                let pivot = <#pivot_type as ::dirtybase_common::db::table_model::TableModel>::make_query_builder();

                let mut relation = ::dirtybase_common::db::repo_relation::Relation::new(
                    ::dirtybase_common::db::repo_relation::RelationType::HasManyThrough{ query, pivot },
                    |relation: ::dirtybase_common::db::repo_relation::Relation,
                     rows: &Vec<::dirtybase_common::db::types::StructuredColumnAndValue>,
                     join_values: &mut ::std::collections::HashMap<String,::std::collections::HashMap<u64,::dirtybase_common::db::field_values::FieldValue>>
                    | {
                        let (mut query, mut pivot) = relation.rel_type().builders();

                        query.select_multiple(&<#foreign_type as ::dirtybase_common::db::table_model::TableModel>::table_query_col_aliases(None));

                        let parent_col_name = #parent_col.to_string();
                        if let Some(mut pivot) = pivot {
                            pivot.select(#pivot_through_col);
                            if join_values.get(&parent_col_name).is_none() {
                                let mut values = ::std::collections::HashMap::new();
                                let prefix = <#parent as ::dirtybase_common::db::table_model::TableModel>::table_name();
                                for a_row in rows {
                                    let mut hash = 0_u64;
                                    let fields = a_row.fields_ref();
                                    if let Some(::dirtybase_common::db::field_values::FieldValue::U64(h)) = fields.get("__hash").cloned() {
                                        hash = h;
                                    }
                                    if let Some(::dirtybase_common::db::field_values::FieldValue::Object(data)) = fields.get(prefix) {
                                        if let Some(v) = data.get(&parent_col_name).cloned() {
                                            values.insert(hash, v);
                                        }
                                    }
                                }
                                join_values.insert(parent_col_name.clone(), values);
                            }

                            let values = join_values.get(&parent_col_name).cloned().unwrap().into_values().collect::<Vec<
                                ::dirtybase_common::db::field_values::FieldValue
                            >>();
                            pivot.is_in(#foreign_col, values);
                            query.is_in_query(
                                    <#foreign_type as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(#through_col),
                                    pivot
                            );
                        }
                        let pivot_data = format!("{}_pivot", #name);
                        query.inner_join_table_and_select::<#foreign_type,#pivot_type>(#through_col, #pivot_through_col, Some(&pivot_data));
                        ::dirtybase_common::db::repo_relation::RelationProcessor::new(query, parent_col_name, pivot_data, #foreign_col.to_string())
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

            pub fn #method_name_where<F>(&mut self,mut callback: F) -> &mut Self
             where F: FnMut(&mut ::dirtybase_common::db::repo_relation::Relation)
            {
                self.#when_method_name(|relation|{
                    #call_callback
                    #trash_condition
                })
            }
        };

        list.push(token);

        if !attribute.no_soft_delete {
            list.push(quote! {
                    pub fn #trashed_method_name(&mut self,) -> &mut Self {
                        self.#trashed_method_name_where(#empty_callback)
                    }

                    pub fn #trashed_method_name_where<F>(&mut self,mut callback: F) -> &mut Self 
                        where F: FnMut(&mut ::dirtybase_common::db::repo_relation::Relation)
                    {
                        self.#when_method_name(|relation| {
                            #call_callback
                        })
                    }
                }
            );

            list.push(quote! {
                    pub fn #with_only_trashed_method_name_where<F>(&mut self,mut callback: F) -> &mut Self 
                        where F: FnMut(&mut ::dirtybase_common::db::repo_relation::Relation)
                    {
                        self.#when_method_name(|relation| {
                            #call_callback
                            relation.rel_type_mut().query_mut().is_not_null(
                                    <#foreign_type as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                                        <#foreign_type as ::dirtybase_common::db::table_model::TableModel>::deleted_at_column().as_ref().unwrap()
                                    )
                                );
                            }
                        )
                    }
                    pub fn #with_only_trashed_method_name(&mut self,) -> &mut Self {
                        self.#with_only_trashed_method_name_where(#empty_callback)
                    }

                }
            );
        }
    }
}

pub(crate) fn build_entity_append(
    attr: &DirtybaseAttributes,
    list: &mut Vec<TokenStream>,
) {
    let name = &attr.name;
    let name_ident = format_ident!("{}", name);
    let foreign_type = format_ident!("{}", attr.the_type);

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
        // Note: rows_rel_map, row_hash ane row_entity are from the `get` method of the repo instance
        if let Some(rows) = rows_rel_map.get_mut(#name) {
            if let Some(related_rows) = rows.remove(row_hash)  {
                #body
            }
        }
    };
    list.push(token);
}
