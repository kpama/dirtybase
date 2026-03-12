use serde::{Deserialize, Serialize};

use crate::db::base::order_by_builder::Direction;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PaginateBuilder {
    limit: usize,
    order: Vec<(String, Direction)>,
    offset: usize,
}

impl PaginateBuilder {
    pub fn new(order_column: &str, offset: usize, limit: usize) -> Self {
        Self {
            order: vec![(order_column.to_string(), Direction::ASC)],
            limit,
            offset,
        }
    }

    pub fn order(&mut self) -> &mut Vec<(String, Direction)> {
        &mut self.order
    }

    pub fn add_order(&mut self, column: &str, direction: Direction) -> &mut Self {
        self.order.push((column.to_string(), direction));
        self
    }

    pub fn set_desc(&mut self) -> &mut Self {
        for order in &mut self.order {
            order.1 = super::order_by_builder::Direction::DESC;
            break;
        }
        self
    }

    pub fn set_asc(&mut self) -> &mut Self {
        for order in &mut self.order {
            order.1 = super::order_by_builder::Direction::ASC;
            break;
        }
        self
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
    pub fn set_offset(&mut self, offset: usize) -> &mut Self {
        self.offset = offset;
        self
    }

    pub fn limit(&self) -> usize {
        self.limit
    }
}

impl Default for PaginateBuilder {
    fn default() -> Self {
        Self::new("id", 0, 25)
    }
}

pub struct PaginateResult<T> {
    page: PaginateBuilder,
    data: Result<Vec<T>, anyhow::Error>,
}

impl<T> PaginateResult<T> {
    pub fn new(page: PaginateBuilder, data: Result<Vec<T>, anyhow::Error>) -> Self {
        Self { page, data }
    }

    pub fn page(&self) -> PaginateBuilder {
        self.page.clone()
    }
    pub fn page_ref(&self) -> &PaginateBuilder {
        &self.page
    }

    pub fn data_ref(&self) -> &Result<Vec<T>, anyhow::Error> {
        &self.data
    }

    pub fn parts(self) -> (PaginateBuilder, Result<Vec<T>, anyhow::Error>) {
        (self.page, self.data)
    }
}
