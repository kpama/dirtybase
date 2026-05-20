use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    marker::PhantomData,
    sync::Arc,
};

use crate::db::{
    TableModel,
    base::{
        cursor_builder::{CursorBuilder, CursorResult},
        manager::Manager,
        query::QueryBuilder,
    },
    field_values::FieldValue,
    types::{FromColumnAndValue, StructuredColumnAndValue},
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
    is_pivot_soft_deletable: bool,
    process: Option<
        Arc<
            Box<
                dyn Fn(
                        Self,
                        &[T],
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
            is_pivot_soft_deletable: self.is_pivot_soft_deletable.clone(),
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
            &[T],
            &mut HashMap<String, HashMap<u64, FieldValue>>,
        ) -> RelationProcessor
        + Send
        + Sync
        + 'static,
    ) -> Self {
        Self {
            rel_type,
            is_pivot_soft_deletable: false,
            process: Some(Arc::new(Box::new(process))),
        }
    }

    pub fn is_pivot_soft_deletable(&self) -> bool {
        self.is_pivot_soft_deletable
    }

    pub fn set_pivot_soft_deletable(&mut self, value: bool) -> &mut Self {
        self.is_pivot_soft_deletable = value;
        self
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
        // Database Manager
        manager: &Manager,
        // The parent raw rows
        rows: &[T],
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
            Ok(rel_list) => {
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
            Err(e) => return Err(e),
        }

        Ok(())
    }

    pub fn build_cursor_paginator<R: TableModel + FromColumnAndValue>(
        mut self,
        // Database Manager
        manager: &Manager,
        // The parent raw rows
        rows: &[T],
        //  Values from the parent rows
        join_field_values: &mut HashMap<String, HashMap<u64, FieldValue>>,
    ) -> RelationCursorPaginator<R> {
        let process = self
            .process
            .take()
            .expect("could not get relation processor");

        let processor = (process)(self, rows, join_field_values);

        RelationCursorPaginator {
            processor,
            manager: manager.clone(),
            next_cursor: None,
            _phantom_data: PhantomData::default(),
        }
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

pub struct RelationCursorPaginator<T> {
    manager: Manager,
    processor: RelationProcessor,
    next_cursor: Option<CursorBuilder>,
    _phantom_data: PhantomData<T>,
}

impl<T: TableModel + FromColumnAndValue> RelationCursorPaginator<T> {
    pub async fn next(&mut self) -> CursorResult<T> {
        self.fetch(None).await
    }

    pub async fn fetch(&mut self, cursor: Option<CursorBuilder>) -> CursorResult<T> {
        let cursor = if let Some(cursor) = cursor {
            cursor
        } else {
            self.next_cursor
                .as_ref()
                .cloned()
                .unwrap_or_else(|| CursorBuilder::new(&T::prefix_with_tbl(T::id_column()), None))
        };

        let query = self.processor.query.clone();

        let result = self
            .manager
            .execute_query(query)
            .cursor_paginate_to::<T>(cursor)
            .await;

        self.next_cursor = result.next();

        result
    }
}
