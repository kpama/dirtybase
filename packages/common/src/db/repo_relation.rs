use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    sync::Arc,
};

use crate::db::{
    base::{manager::Manager, query::QueryBuilder},
    field_values::FieldValue,
    types::StructuredColumnAndValue,
};

#[derive(Clone)]
pub enum RelationType {
    HasOne {
        query: QueryBuilder,
    },
    BelongsTo {
        query: QueryBuilder,
    },
    HasMany {
        query: QueryBuilder,
    },
    HasOneThrough {
        query: QueryBuilder,
        pivot: QueryBuilder,
    },
    HasManyThrough {
        query: QueryBuilder,
        pivot: QueryBuilder,
    },
    BelongsToMany {
        query: QueryBuilder,
    },
    MorphOne {
        query: QueryBuilder,
        // pivot: QueryBuilder,
    },
    MorphMany {
        query: QueryBuilder,
        // pivot: QueryBuilder,
    },
}

impl Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _type = match self {
            Self::BelongsTo { query: _ } => "belongs_to",
            Self::BelongsToMany { query: _ } => "belongs_to_many",
            Self::HasOneThrough { query: _, pivot: _ } => "has_one_through",
            Self::HasMany { query: _ } => "has_many",
            Self::HasManyThrough { query: _, pivot: _ } => "has_many_through",
            Self::HasOne { query: _ } => "has_one",
            Self::MorphMany { query: _ } => "morph_many",
            Self::MorphOne { query: _ } => "morph_one",
        };

        write!(f, "{}", _type)
    }
}

impl RelationType {
    pub fn builders(self) -> (QueryBuilder, Option<QueryBuilder>) {
        match self {
            Self::BelongsTo { query } => (query, None),
            Self::BelongsToMany { query } => (query, None),
            Self::HasManyThrough { query, pivot } => (query, Some(pivot)),
            Self::HasOneThrough { query, pivot } => (query, Some(pivot)),
            Self::HasMany { query } => (query, None),
            Self::HasOne { query } => (query, None),
            Self::MorphMany { query } => (query, None),
            Self::MorphOne { query } => (query, None),
        }
    }

    pub fn query_mut(&mut self) -> &mut QueryBuilder {
        match self {
            Self::BelongsTo { query } => query,
            Self::BelongsToMany { query } => query,
            Self::HasManyThrough { query, pivot: _ } => query,
            Self::HasOneThrough { query, pivot: _ } => query,
            Self::HasMany { query } => query,
            Self::HasOne { query } => query,
            Self::MorphMany { query } => query,
            Self::MorphOne { query } => query,
        }
    }

    pub fn pivot_mut(&mut self) -> Option<&mut QueryBuilder> {
        match self {
            Self::BelongsTo { query: _ } => None,
            Self::BelongsToMany { query: _ } => None,
            Self::HasManyThrough { query: _, pivot } => Some(pivot),
            Self::HasOneThrough { query: _, pivot } => Some(pivot),
            Self::HasMany { query: _ } => None,
            Self::HasOne { query: _ } => None,
            Self::MorphMany { query: _ } => None,
            Self::MorphOne { query: _ } => None,
        }
    }
}

pub struct Relation<T> {
    rel_type: RelationType,
    process: Option<
        Arc<
            Box<
                dyn Fn(
                        Self,
                        &HashMap<u64, T>,
                        &mut HashMap<String, HashMap<u64, FieldValue>>,
                    ) -> RelationProcessor
                    + Sync
                    + Send,
            >,
        >,
    >,
}

impl<T> Clone for Relation<T> {
    fn clone(&self) -> Self {
        Self {
            rel_type: self.rel_type.clone(),
            process: self.process.clone(),
        }
    }
}

impl<T> Debug for Relation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<T> Display for Relation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.rel_type)
    }
}

impl<T> Relation<T> {
    pub fn new(
        rel_type: RelationType,
        process: impl Fn(
            Self,
            &HashMap<u64, T>,
            &mut HashMap<String, HashMap<u64, FieldValue>>,
        ) -> RelationProcessor
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            rel_type,
            process: Some(Arc::new(Box::new(process))),
        }
    }

    pub fn rel_type_mut(&mut self) -> &mut RelationType {
        &mut self.rel_type
    }

    pub fn query_mut(&mut self) -> &mut QueryBuilder {
        self.rel_type_mut().query_mut()
    }

    pub fn pivot_mut(&mut self) -> Option<&mut QueryBuilder> {
        self.rel_type_mut().pivot_mut()
    }

    pub fn rel_type(self) -> RelationType {
        self.rel_type
    }

    /// Fetches and process the relation  data
    ///
    pub async fn process(
        mut self,
        // The relation name
        name: &str,
        // Db Manager
        manager: &Manager,
        // The parent raw rows
        rows: &HashMap<u64, T>,
        //  Values from the parent rows
        join_field_values: &mut HashMap<String, HashMap<u64, FieldValue>>,
        // Built relation data
        rows_rel_map: &mut HashMap<String, HashMap<u64, Vec<StructuredColumnAndValue>>>,
    ) -> Result<(), anyhow::Error> {
        if rows.is_empty() {
            return Ok(());
        }

        let process = self
            .process
            .take()
            .expect("could not get relation processor");

        let RelationProcessor {
            query,
            child_col_name,
            child_field_prefix,
            parent_col_name,
        } = (process)(self, rows, join_field_values);

        match manager.execute_query(query).all().await {
            Ok(Some(rel_list)) => {
                for a_row in rel_list {
                    let mut belongs_to_hash = Vec::new();

                    if let Some(FieldValue::Object(obj)) = a_row.get(&child_field_prefix) {
                        if let Some(value) = obj.get(&child_col_name) {
                            if let Some(kv) = join_field_values.get(&parent_col_name) {
                                for (hash, val) in kv {
                                    if val == value {
                                        belongs_to_hash.push(*hash);
                                    }
                                }
                            }
                        }
                    }

                    if !belongs_to_hash.is_empty() {
                        if rows_rel_map.get(name).is_none() {
                            rows_rel_map
                                .insert(name.to_string(), ::std::collections::HashMap::new());
                        }

                        for hash in belongs_to_hash {
                            if rows_rel_map.get(name).unwrap().get(&hash).is_none() {
                                rows_rel_map.get_mut(name).unwrap().insert(hash, Vec::new());
                            }
                            rows_rel_map
                                .get_mut(name)
                                .unwrap()
                                .get_mut(&hash)
                                .unwrap()
                                .push(a_row.clone());
                        }
                    }
                }
            }
            Ok(None) => (),
            Err(e) => return Err(e),
        }

        Ok(())
    }
}

pub struct RelationProcessor {
    pub parent_col_name: String,
    pub child_field_prefix: String,
    pub child_col_name: String,
    pub query: QueryBuilder,
}

impl RelationProcessor {
    pub fn new(
        query: QueryBuilder,
        parent_col_name: String,
        child_field_prefix: String,
        child_col_name: String,
    ) -> Self {
        Self {
            query,
            parent_col_name,
            child_field_prefix,
            child_col_name,
        }
    }
}
