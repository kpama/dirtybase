use std::collections::HashMap;

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use axum_extra::extract::Query;
use serde::{Deserialize, Serialize};

use crate::db::base::order_by_builder::{Direction, LimitBuilder, OffsetBuilder, OrderByBuilder};

/// # Paginate Builder
///
/// Uses `offset`, `limit` and `sorting`
///
/// Extract an instance from the current HTTP request
///```rust,no_run
/// async handler(page: PaginateBuilder) {...}
/// ```
///
/// Attributes that are extracted from the current HTTP request:
///
/// - `_limit` : An unsigned numeric value
/// - `_offset` : An unsigned number value
/// - `_sort` : One or more string values separated by comma, `,`. Prepending a minus,`-id`, means sort DESCENDING and
///            plus, `+id`, means ASCENDING. Ascending is the default order means ASCENDING. Ascending is the default ordering.
///
/// Example: `_limit=20&_offset=4&_sort=-id`
///
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PaginateBuilder {
    #[serde(flatten)]
    pub(crate) limit: LimitBuilder,
    #[serde(flatten)]
    pub(crate) order: OrderByBuilder,
    #[serde(flatten)]
    pub(crate) offset: OffsetBuilder,
    #[serde(flatten)]
    total: usize,
}

impl PaginateBuilder {
    pub fn new(order_column: &str, offset: usize, limit: usize) -> Self {
        Self {
            order: OrderByBuilder::new_asc(order_column),
            limit: LimitBuilder::new(limit),
            offset: OffsetBuilder::new(offset),
            total: 0,
        }
    }

    pub fn order(&self) -> &OrderByBuilder {
        &self.order
    }

    pub fn add_order(&mut self, column: &str, direction: Direction) -> &mut Self {
        if direction == Direction::ASC {
            self.order.add_asc(column);
        } else {
            self.order.add_desc(column);
        }
        self
    }

    pub fn set_order(&mut self, order: OrderByBuilder) -> &mut Self {
        self.order = order;
        self
    }

    pub fn offset(&self) -> &OffsetBuilder {
        &self.offset
    }
    pub fn set_offset(&mut self, offset: usize) -> &mut Self {
        self.offset.set_offset(offset);
        self
    }

    pub fn set_limit(&mut self, limit: usize) -> &mut Self {
        self.limit.set_limit(limit);
        self
    }

    pub fn set_total(&mut self, total: usize) -> &mut Self {
        self.total = total;
        self
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn limit(&self) -> &LimitBuilder {
        &self.limit
    }

    pub fn as_query_string(&self) -> String {
        let mut query = Vec::new();
        query.push(format!("_limit={}", self.limit.limit));
        query.push(format!("_offset={}", self.offset.offset));
        query.push(format!("_sort={}", self.order().as_uri_query()));
        query.join("&")
    }
}

impl Default for PaginateBuilder {
    fn default() -> Self {
        Self::new("id", 0, 100)
    }
}

pub struct PaginateResult<T> {
    previous: Option<PaginateBuilder>,
    next: Option<PaginateBuilder>,
    data: Result<Vec<T>, anyhow::Error>,
}

impl<T> PaginateResult<T> {
    pub fn new(
        data: Result<Vec<T>, anyhow::Error>,
        next: Option<PaginateBuilder>,
        previous: Option<PaginateBuilder>,
    ) -> Self {
        Self {
            next,
            data,
            previous,
        }
    }

    pub fn next(&self) -> Option<PaginateBuilder> {
        self.next.clone()
    }
    pub fn next_ref(&self) -> Option<&PaginateBuilder> {
        self.next.as_ref()
    }

    pub fn previous(&self) -> Option<PaginateBuilder> {
        self.previous.clone()
    }

    pub fn previous_ref(&self) -> Option<&PaginateBuilder> {
        self.previous.as_ref()
    }

    pub fn data_ref(&self) -> &Result<Vec<T>, anyhow::Error> {
        &self.data
    }

    /// Returns tuple of (data, next, previous)
    pub fn parts(
        self,
    ) -> (
        Result<Vec<T>, anyhow::Error>,
        Option<PaginateBuilder>,
        Option<PaginateBuilder>,
    ) {
        (self.data, self.next, self.previous)
    }
}

impl<S> FromRequestParts<S> for PaginateBuilder
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let mut builder = Self::default();
        match Query::<HashMap<String, String>>::try_from_uri(&parts.uri) {
            Ok(kv) => {
                // Limit
                if let Some(v) = kv.get("_limit")
                    && let Ok(limit) = v.parse::<usize>()
                {
                    builder.set_limit(limit);
                }
                // Offset
                if let Some(v) = kv.get("_offset")
                    && let Ok(offset) = v.parse::<usize>()
                {
                    builder.set_offset(offset);
                }

                // Sort
                if let Some(sort) = kv.get("_sort") {
                    let list = sort
                        .split(',')
                        .into_iter()
                        .filter(|entry| entry.trim().len() > 0 && *entry != "-" && *entry != "+")
                        .map(|entry| {
                            if entry.starts_with('-') {
                                (entry.trim_matches('-').to_string(), Direction::DESC)
                            } else {
                                (entry.trim_matches('+').to_string(), Direction::ASC)
                            }
                        })
                        .collect::<Vec<(String, Direction)>>();
                    if list.len() > 0 {
                        builder.set_order(OrderByBuilder::from(list));
                    } else {
                        tracing::warn!("query string sort list is empty");
                    }
                }
            }
            Err(e) => tracing::error!("error building paginator from query: {}", e),
        }
        Ok(builder)
    }
}
