use std::fmt::{Debug, Display, write};

use serde::{Deserialize, Serialize};

use crate::db::{
    base::{
        helper,
        order_by_builder::{LimitBuilder, OrderByBuilder},
    },
    field_values::FieldValue,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CursorBuilder {
    col: String,
    last: Option<FieldValue>,
    limit: LimitBuilder,
    order: OrderByBuilder,
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
    cursor: CursorBuilder,
    data: Result<Vec<T>, anyhow::Error>,
}

impl<T> CursorResult<T> {
    pub fn new(cursor: CursorBuilder, data: Result<Vec<T>, anyhow::Error>) -> Self {
        Self { cursor, data }
    }

    pub fn cursor(&self) -> CursorBuilder {
        self.cursor.clone()
    }

    pub fn cursor_ref(&self) -> &CursorBuilder {
        &self.cursor
    }

    pub fn data_ref(&self) -> &Result<Vec<T>, anyhow::Error> {
        &self.data
    }

    pub fn parts(self) -> (CursorBuilder, Result<Vec<T>, anyhow::Error>) {
        (self.cursor, self.data)
    }
}
