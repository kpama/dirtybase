use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    ASC,
    DESC,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ASC => write!(f, "ASC"),
            Self::DESC => write!(f, "DESC"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderByBuilder {
    pub(crate) order: Vec<(String, Direction)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LimitBuilder {
    pub(crate) limit: usize,
}

impl LimitBuilder {
    pub fn new(limit: usize) -> Self {
        Self { limit }
    }

    pub fn set_limit(&mut self, limit: usize) -> &mut Self {
        self.limit = limit;
        self
    }

    pub fn limit(&self) -> usize {
        self.limit
    }
}

impl Default for LimitBuilder {
    fn default() -> Self {
        Self::new(25)
    }
}

impl Display for LimitBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " LIMIT {}", &self.limit)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OffsetBuilder {
    pub(crate) offset: usize,
}

impl Default for OffsetBuilder {
    fn default() -> Self {
        Self { offset: 0 }
    }
}

impl OffsetBuilder {
    pub fn new(offset: usize) -> Self {
        Self { offset }
    }

    pub fn set_offset(&mut self, offset: usize) -> &mut Self {
        self.offset = offset;
        self
    }
}

impl Display for OffsetBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " OFFSET {}", &self.offset)
    }
}

impl Default for OrderByBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderByBuilder {
    pub fn new() -> Self {
        Self { order: Vec::new() }
    }

    pub fn new_asc<C: ToString>(column: C) -> Self {
        Self {
            order: vec![(column.to_string(), Direction::ASC)],
        }
    }

    pub fn new_desc<C: ToString>(column: C) -> Self {
        Self {
            order: vec![(column.to_string(), Direction::DESC)],
        }
    }

    pub fn add_asc<C: ToString>(&mut self, column: C) -> &mut Self {
        self.order.push((column.to_string(), Direction::ASC));
        self
    }

    pub fn add_desc<C: ToString>(&mut self, column: C) -> &mut Self {
        self.order.push((column.to_string(), Direction::DESC));
        self
    }

    pub fn as_clause(&self) -> String {
        let orders = self
            .order
            .iter()
            .map(|entry| format!("{} {}", entry.0, entry.1))
            .collect::<Vec<String>>()
            .join(",");

        format!("ORDER BY {orders}",)
    }

    pub fn as_uri_query(&self) -> String {
        self.order
            .iter()
            .map(|entry| {
                if entry.1 == Direction::DESC {
                    format!("-{}", entry.0)
                } else {
                    format!("{}", entry.0)
                }
            })
            .collect::<Vec<String>>()
            .join(",")
    }
}

impl From<Vec<(String, Direction)>> for OrderByBuilder {
    fn from(order: Vec<(String, Direction)>) -> Self {
        Self { order }
    }
}

impl Display for OrderByBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_clause())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_order_asc_a_field() {
        let mut order = OrderByBuilder::new();
        order.add_asc("a");

        assert_eq!(order.to_string(), "ORDER BY a ASC");
    }

    #[test]
    fn test_order_desc_a_field() {
        let mut order = OrderByBuilder::new();
        order.add_desc("a");

        assert_eq!(order.to_string(), "ORDER BY a DESC");
    }

    #[test]
    fn test_multi_order_asc_a_field() {
        let mut order = OrderByBuilder::new();
        order.add_asc("a");
        order.add_asc("b");
        order.add_asc("c");

        assert_eq!(order.to_string(), "ORDER BY a ASC,b ASC,c ASC");
    }

    #[test]
    fn test_multi_order_desc_a_field() {
        let mut order = OrderByBuilder::new();
        order.add_desc("a");
        order.add_desc("b");
        order.add_desc("c");

        assert_eq!(order.to_string(), "ORDER BY a DESC,b DESC,c DESC");
    }

    #[test]
    fn test_mix_order() {
        let mut order = OrderByBuilder::new();
        order.add_desc("a");
        order.add_asc("b");
        order.add_desc("c");
        order.add_asc("d");

        assert_eq!(order.to_string(), "ORDER BY a DESC,b ASC,c DESC,d ASC");
    }
}
