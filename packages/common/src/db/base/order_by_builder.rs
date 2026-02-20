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
    pub(crate) orders: Vec<(String, Direction)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LimitBuilder {
    pub(crate) limit: usize,
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

impl Display for OffsetBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ", OFFSET {}", &self.offset)
    }
}

impl Default for OrderByBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OrderByBuilder {
    pub fn new() -> Self {
        Self { orders: Vec::new() }
    }

    pub fn new_asc<C: ToString>(column: C) -> Self {
        Self {
            orders: vec![(column.to_string(), Direction::ASC)],
        }
    }

    pub fn new_desc<C: ToString>(column: C) -> Self {
        Self {
            orders: vec![(column.to_string(), Direction::DESC)],
        }
    }

    pub fn asc<C: ToString>(&mut self, column: C) -> &mut Self {
        self.orders.push((column.to_string(), Direction::ASC));
        self
    }

    pub fn desc<C: ToString>(&mut self, column: C) -> &mut Self {
        self.orders.push((column.to_string(), Direction::DESC));
        self
    }

    pub fn as_clause(&self) -> String {
        let orders = self
            .orders
            .iter()
            .map(|entry| format!("{} {}", entry.0, entry.1))
            .collect::<Vec<String>>()
            .join(",");

        format!("ORDER BY {orders}",)
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
        order.asc("a");

        assert_eq!(order.to_string(), "ORDER BY a ASC");
    }

    #[test]
    fn test_order_desc_a_field() {
        let mut order = OrderByBuilder::new();
        order.desc("a");

        assert_eq!(order.to_string(), "ORDER BY a DESC");
    }

    #[test]
    fn test_multi_order_asc_a_field() {
        let mut order = OrderByBuilder::new();
        order.asc("a");
        order.asc("b");
        order.asc("c");

        assert_eq!(order.to_string(), "ORDER BY a ASC,b ASC,c ASC");
    }

    #[test]
    fn test_multi_order_desc_a_field() {
        let mut order = OrderByBuilder::new();
        order.desc("a");
        order.desc("b");
        order.desc("c");

        assert_eq!(order.to_string(), "ORDER BY a DESC,b DESC,c DESC");
    }

    #[test]
    fn test_mix_order() {
        let mut order = OrderByBuilder::new();
        order.desc("a");
        order.asc("b");
        order.desc("c");
        order.asc("d");

        assert_eq!(order.to_string(), "ORDER BY a DESC,b ASC,c DESC,d ASC");
    }
}
