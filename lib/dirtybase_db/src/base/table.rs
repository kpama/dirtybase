use crate::base::to_fk_column;

use super::{
    column::{BaseColumn, ColumnType, RelationType},
    index::{IndexProp, IndexType},
    query::QueryBuilder,
    user_table::user_table_name,
};
use std::fmt::Debug;

#[derive(Debug)]
pub struct BaseTable {
    pub name: String,
    pub new_name: Option<String>,
    pub columns: Vec<BaseColumn>,
    pub is_new: bool,
    pub view_query: Option<QueryBuilder>,
    pub indexes: Option<Vec<IndexType>>,
}

impl BaseTable {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            new_name: None,
            columns: Vec::new(),
            is_new: true,
            view_query: None,
            indexes: None,
        }
    }

    /// Rename an existing table
    pub fn rename(&mut self, new_name: &str) -> &mut Self {
        self.new_name = Some(new_name.to_owned());
        self
    }

    pub fn set_is_new(&mut self, new: bool) -> &mut Self {
        self.is_new = new;
        self
    }

    pub fn column(
        &mut self,
        name: &'static str,
        callback: impl FnOnce(&mut BaseColumn),
    ) -> &mut BaseColumn {
        let mut column = BaseColumn::new(name, ColumnType::String(255));

        callback(&mut column);
        self.columns.push(column);

        self.columns.last_mut().unwrap()
    }

    pub fn boolean(&mut self, name: &'static str) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::Boolean);
        })
    }

    pub fn char(&mut self, name: &'static str, length: usize) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::Char(length));
        })
    }

    pub fn date(&mut self, name: &'static str) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::Date);
        })
    }

    pub fn file(&mut self, name: &'static str, relation_type: RelationType) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::File(relation_type));
        })
    }

    pub fn float(&mut self, name: &'static str) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::Float);
        })
    }

    pub fn integer(&mut self, name: &'static str) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::Integer);
        })
    }

    pub fn json(&mut self, name: &'static str) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::Json);
        })
    }

    pub fn number(&mut self, name: &'static str) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::Number);
        })
    }

    pub fn relation(
        &mut self,
        name: &'static str,
        relation_type: RelationType,
        table_name: &'static str,
    ) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::Relation {
                relation_type,
                table_name: table_name.to_owned(),
            });
        })
    }

    pub fn select(&mut self, name: &'static str, relation_type: RelationType) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::Select(relation_type));
        })
    }
    pub fn string(&mut self, name: &'static str) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::String(255));
        })
    }

    pub fn sized_string(&mut self, name: &'static str, length: usize) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::String(length));
        })
    }

    pub fn text(&mut self, name: &'static str) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::String(512));
        })
    }

    pub fn uuid(&mut self, name: &'static str) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::Uuid);
        })
    }

    pub fn ulid(&mut self, name: &'static str) -> &mut BaseColumn {
        self.char(name, 26)
    }

    pub fn id_set(&mut self) {
        self.id(Some("internal_id"));
        self.ulid("id").set_is_unique(true).set_is_nullable(false);
    }

    pub fn id(&mut self, name: Option<&'static str>) -> &mut BaseColumn {
        self.column(name.unwrap_or("id"), |column| {
            column.set_type(ColumnType::AutoIncrementId);
        })
    }

    pub fn created_at(&mut self) -> &mut BaseColumn {
        self.date("created_at")
            .set_is_nullable(false)
            .default_is_created_at()
    }

    pub fn updated_at(&mut self) -> &mut BaseColumn {
        self.date("updated_at")
            .set_is_nullable(false)
            .default_is_updated_at()
    }

    pub fn timestamps(&mut self) {
        let mut _created_at = self.created_at();
        let mut _updated_at = self.updated_at();
    }

    pub fn blame(&mut self) {
        let user_table_name = &user_table_name();
        let mut _creator = self
            .ulid("creator")
            .set_is_nullable(false)
            .references_without_cascade_delete(user_table_name, "id");

        let mut _editor = self
            .ulid("editor")
            .set_is_nullable(true)
            .references_without_cascade_delete(user_table_name, "id");
    }

    pub fn soft_deletable(&mut self) -> &mut BaseColumn {
        self.date("deleted_at").set_is_nullable(true)
    }

    pub fn is_new(&self) -> bool {
        self.is_new
    }

    pub fn columns(&self) -> &Vec<BaseColumn> {
        &self.columns
    }

    pub fn column_string(
        &mut self,
        name: String,
        callback: impl FnOnce(&mut BaseColumn),
    ) -> &mut BaseColumn {
        let mut column = BaseColumn::new(&name, ColumnType::String(255));

        callback(&mut column);
        self.columns.push(column);

        self.columns.last_mut().unwrap()
    }

    // pub fn unique_index(&mut self, columns: &[&str]) -> &mut Self {
    //     if self.indexes.is_none() {
    //         self.indexes = Some(Vec::new());
    //     }

    //     if let Some(indexes) = &mut self.indexes {
    //         indexes.push(IndexType::Unique(
    //             columns.iter().map(|c| c.to_string()).collect(),
    //         ));
    //     }

    //     self
    // }

    pub fn primary_index(&mut self, columns: &[&str]) -> &mut Self {
        if self.indexes.is_none() {
            self.indexes = Some(Vec::new());
        }

        if let Some(indexes) = &mut self.indexes {
            indexes.push(IndexType::Primary(IndexProp::new(columns, false)));
        }
        self
    }
}
