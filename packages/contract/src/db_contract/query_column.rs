use super::base::{aggregate::Aggregate, query::QueryBuilder};

#[derive(Debug, Clone, PartialEq)]
pub enum QueryColumnName {
    Name(String),
    SubQuery(Box<QueryBuilder>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryColumn {
    name: QueryColumnName,
    table: Option<String>,
    alias: Option<String>,
    aggregate: Option<Aggregate>,
}

impl QueryColumn {
    pub fn new<T: Into<QueryColumnName>>(
        name: T,
        table: Option<&str>,
        alias: Option<&str>,
    ) -> Self {
        Self {
            name: name.into(),
            alias: alias.map(String::from),
            table: table.map(String::from),
            aggregate: None,
        }
    }

    pub fn name(&self) -> &QueryColumnName {
        &self.name
    }

    pub fn table(&self) -> &Option<String> {
        &self.table
    }

    pub fn alias(&self) -> &Option<String> {
        &self.alias
    }

    pub fn set_alias(&mut self, alias: &str) {
        self.alias = Some(alias.to_string())
    }

    pub fn set_table(&mut self, table: &str) {
        self.table = Some(table.to_string());
    }

    pub fn set_aggregate(&mut self, agg: Aggregate) {
        self.aggregate = Some(agg);
    }

    pub fn aggregate(&self) -> &Option<Aggregate> {
        &self.aggregate
    }
}

impl<T: ToString> From<T> for QueryColumnName {
    fn from(value: T) -> Self {
        Self::Name(value.to_string())
    }
}

impl From<QueryBuilder> for QueryColumnName {
    fn from(value: QueryBuilder) -> Self {
        Self::SubQuery(Box::new(value))
    }
}

impl From<QueryColumnName> for QueryColumn {
    fn from(name: QueryColumnName) -> Self {
        Self::new(name, None, None)
    }
}

impl From<String> for QueryColumn {
    fn from(value: String) -> Self {
        QueryColumnName::from(value).into()
    }
}
impl From<&str> for QueryColumn {
    fn from(value: &str) -> Self {
        QueryColumnName::from(value.to_string()).into()
    }
}

impl<T: Into<QueryColumn> + Clone> From<&T> for QueryColumn {
    fn from(value: &T) -> Self {
        value.clone().into()
    }
}

// (column, table)
impl<C: Into<QueryColumnName>, T: ToString> From<(C, T)> for QueryColumn {
    fn from(value: (C, T)) -> Self {
        Self {
            name: value.0.into(),
            table: Some(value.1.to_string()),
            alias: None,
            aggregate: None,
        }
    }
}

// (column, table, alias)
impl<C: Into<QueryColumnName>, T: ToString, A: ToString> From<(C, Option<T>, Option<A>)>
    for QueryColumn
{
    fn from(value: (C, Option<T>, Option<A>)) -> Self {
        Self {
            name: value.0.into(),
            table: value.1.map(|t| t.to_string()),
            alias: value.2.map(|a| a.to_string()),
            aggregate: None,
        }
    }
}
