use std::sync::Arc;

use busybody::{helpers, Injectable, ServiceContainer};

use crate::{base::manager::Manager, field_values::FieldValue, ConnectionPoolManager};

#[derive(Default)]
pub struct DB {
    service: Option<Arc<ServiceContainer>>,
}

impl DB {
    pub fn new(service: Option<Arc<ServiceContainer>>) -> Self {
        Self { service }
    }

    pub async fn insert<T: Into<FieldValue>>(&self, sql: &str, values: &[T]) {}
    pub async fn update(&self) {}
    pub async fn delete(&self) {}
    pub async fn statement(&self) {}
    pub async fn unprepared(&self) {}
    pub async fn transaction(&self) {}

    pub fn begin_transaction(&self) {}
}
