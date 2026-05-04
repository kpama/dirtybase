use std::{collections::HashMap, fmt::Debug};

use axum::{
    extract::{FromRequestParts, Query},
    http::{StatusCode, request::Parts},
};
use serde::{Deserialize, Serialize};

use crate::db::{
    base::order_by_builder::{Direction, LimitBuilder, OrderByBuilder},
    field_values::FieldValue,
};

/// # CursorBuilder
///
/// Uses a cursor based on list fetched value for paginating
///
/// Extract an instance from the current HTTP request
///```rust,no_run
/// async handler(cursor: CursorBuilder) {...}
/// ```
/// Attributes that are extracted from the current HTTP request:
///
/// - `_cursor`: A cursor encoded string
/// - `_limit` : An unsigned numeric value
/// - `_col` : The name of the column used
/// - `_last` : Last value from which to start the cursor after
/// - `_sort` : One or more string values separated by comma, `,`. Prepending a minus,`-id`, means sort DESCENDING and
///            plus, `+id`, means ASCENDING. Ascending is the default order means ASCENDING. Ascending is the default ordering.
///
/// Example: `_limit=20&_sort=-id`
/// Example: `_cursor=......`
///
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CursorBuilder {
    pub(crate) col: String,
    pub(crate) last: Option<FieldValue>,
    #[serde(flatten)]
    pub(crate) limit: LimitBuilder,
    #[serde(flatten)]
    pub(crate) order: OrderByBuilder,
}

impl CursorBuilder {
    pub fn new(column: &str, last: Option<FieldValue>) -> Self {
        Self {
            col: column.to_string(),
            order: OrderByBuilder::new_asc(column),
            last,
            ..Default::default()
        }
    }

    pub fn order_by_builder(&mut self) -> &mut OrderByBuilder {
        &mut self.order
    }

    pub fn set_desc(&mut self) -> &mut Self {
        self.order = OrderByBuilder::new_desc(self.column());
        self
    }

    pub fn set_asc(&mut self) -> &mut Self {
        self.order = OrderByBuilder::new_asc(self.column());
        self
    }

    pub fn set_last(&mut self, last: FieldValue) -> &mut Self {
        self.last = Some(last);
        self
    }

    pub fn set_limit(&mut self, limit: usize) -> &mut Self {
        self.limit.limit = limit;
        self
    }

    pub fn limit(&self) -> &LimitBuilder {
        &self.limit
    }

    pub fn last(&self) -> Option<&FieldValue> {
        self.last.as_ref()
    }

    pub fn column(&self) -> &str {
        &self.col
    }

    pub fn order(&self) -> &OrderByBuilder {
        &self.order
    }

    /// Encodes the instance to a base64 string
    pub fn encode(&self) -> String {
        let data = serde_json::to_string(self).expect("could not stringify cursor builder");
        return dirtybase_helper::base64::url_encode(data.as_bytes());
    }

    /// Tries to decode the base64 string to an instance
    pub fn decode(data: &str) -> Result<Self, anyhow::Error> {
        match dirtybase_helper::base64::decode(data) {
            Ok(raw) => serde_json::from_slice(&raw).map_err(|e| anyhow::anyhow!(e)),
            Err(e) => Err(anyhow::anyhow!(e)),
        }
    }

    pub fn as_query_string(&self) -> String {
        let mut query = Vec::new();
        if let Some(last) = &self.last {
            query.push(format!("_last={}", last));
        }
        query.push(format!("_col={}", self.col));
        query.push(format!("_limit={}", self.limit.limit));
        query.push(format!("_sort={}", self.order().as_uri_query()));

        query.join("&")
    }
}

impl Default for CursorBuilder {
    fn default() -> Self {
        Self {
            col: "id".to_string(),
            last: None,
            limit: LimitBuilder { limit: 25 },
            order: OrderByBuilder::new_asc("id"),
        }
    }
}

pub struct CursorResult<T> {
    next: Option<CursorBuilder>,
    previous: Option<CursorBuilder>,
    data: Result<Vec<T>, anyhow::Error>,
}

impl<T> CursorResult<T> {
    pub fn new(
        data: Result<Vec<T>, anyhow::Error>,
        next: Option<CursorBuilder>,
        previous: Option<CursorBuilder>,
    ) -> Self {
        Self {
            next,
            data,
            previous,
        }
    }

    pub fn next(&self) -> Option<CursorBuilder> {
        self.next.clone()
    }
    pub fn next_ref(&self) -> Option<&CursorBuilder> {
        self.next.as_ref()
    }

    pub fn previous(&self) -> Option<CursorBuilder> {
        self.previous.clone()
    }

    pub fn previous_ref(&self) -> Option<&CursorBuilder> {
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
        Option<CursorBuilder>,
        Option<CursorBuilder>,
    ) {
        (self.data, self.next, self.previous)
    }
}

impl<S> FromRequestParts<S> for CursorBuilder
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let mut builder = Self::default();

        match Query::<HashMap<String, String>>::try_from_uri(&parts.uri) {
            Ok(kv) => {
                let mut is_cursor_value = false;
                if let Some(s) = kv.get("_cursor")
                    && let Ok(mut c) = Self::decode(s)
                {
                    is_cursor_value = true;
                    if let Some(last) = c.last.clone() {
                        c.last = Some(last.into());
                    }
                    builder = c;
                }

                // column
                if let Some(col) = kv.get("_col").cloned() {
                    builder.col = col
                }

                // Limit
                if let Some(v) = kv.get("_limit")
                    && let Ok(limit) = v.parse::<usize>()
                {
                    builder.set_limit(limit);
                }

                // last
                if let Some(v) = kv.get("_last") {
                    let value = FieldValue::from(v);
                    if value != FieldValue::NotSet && value != FieldValue::Null {
                        builder.set_last(value);
                    }
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
                        builder.order = OrderByBuilder::from(list);
                    } else {
                        tracing::warn!("query string sort list is empty");
                    }
                }
            }
            Err(e) => tracing::error!("error building cursor paginator from query: {}", e),
        }
        Ok(builder)
    }
}
