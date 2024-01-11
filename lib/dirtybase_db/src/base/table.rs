use crate::{TableEntityTrait, USER_TABLE};

use super::{
    column::{BaseColumn, ColumnType, RelationType},
    helper::to_fk_column,
    index::{IndexProp, IndexType},
    query::QueryBuilder,
};
use std::fmt::Debug;

pub const ULID_STRING_LENGTH: usize = 26;

// Fields
pub const INTERNAL_ID_FIELD: &str = "internal_id";
pub const ID_FIELD: &str = "id";
pub const CREATOR_FIELD: &str = "creator_id";
pub const EDITOR_FIELD: &str = "editor_id";
pub const CREATED_AT_FIELD: &str = "created_at";
pub const UPDATED_AT_FIELD: &str = "updated_at";
pub const DELETED_AT_FIELD: &str = "deleted_at";

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

    pub fn datetime(&mut self, name: &'static str) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::Datetime);
        })
    }

    pub fn timestamp(&mut self, name: &'static str) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::Timestamp);
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
            column.set_type(ColumnType::String(256));
        })
    }

    pub fn sized_string(&mut self, name: &'static str, length: usize) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::String(length));
        })
    }

    pub fn text(&mut self, name: &'static str) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::Text);
        })
    }

    pub fn uuid(&mut self, name: &'static str) -> &mut BaseColumn {
        self.column(name, |column| {
            column.set_type(ColumnType::Uuid);
        })
    }

    pub fn ulid_fk(&mut self, foreign_table: &str, cascade_delete: bool) -> &mut BaseColumn {
        let name = to_fk_column(foreign_table, None);

        self.column_string(name, |column| {
            column.set_type(ColumnType::Char(ULID_STRING_LENGTH));
            if cascade_delete {
                column.references_with_cascade_delete(foreign_table, ID_FIELD);
            } else {
                column.references_without_cascade_delete(foreign_table, ID_FIELD);
            }
        })
    }

    pub fn ulid_table_fk<F: TableEntityTrait>(&mut self, cascade_delete: bool) -> &mut BaseColumn {
        let foreign_table = F::table_name();
        let id = F::id_column().unwrap();
        let name = to_fk_column(foreign_table, Some(id));

        self.column_string(name, |column| {
            column.set_type(ColumnType::Char(ULID_STRING_LENGTH));
            if cascade_delete {
                column.references_with_cascade_delete(foreign_table, id);
            } else {
                column.references_without_cascade_delete(foreign_table, id);
            }
        })
    }

    pub fn id_table_fk<F: TableEntityTrait>(&mut self, cascade_delete: bool) -> &mut BaseColumn {
        let foreign_table = F::table_name();
        let id = F::id_column().unwrap();
        let name = to_fk_column(foreign_table, Some(id));

        self.column_string(name, |column| {
            column.set_type(ColumnType::Integer);
            if cascade_delete {
                column.references_with_cascade_delete(foreign_table, id);
            } else {
                column.references_without_cascade_delete(foreign_table, id);
            }
        })
    }

    pub fn ulid(&mut self, name: &'static str) -> &mut BaseColumn {
        self.char(name, ULID_STRING_LENGTH)
    }

    pub fn id_set(&mut self) {
        self.id(Some(INTERNAL_ID_FIELD));
        self.ulid(ID_FIELD)
            .set_is_unique(true)
            .set_is_nullable(false);
    }

    pub fn id(&mut self, name: Option<&'static str>) -> &mut BaseColumn {
        self.column(name.unwrap_or(ID_FIELD), |column| {
            column.set_type(ColumnType::AutoIncrementId);
        })
    }

    pub fn created_at(&mut self) -> &mut BaseColumn {
        self.timestamp(CREATED_AT_FIELD)
            .set_is_nullable(false)
            .default_is_created_at()
    }

    pub fn updated_at(&mut self) -> &mut BaseColumn {
        self.timestamp(UPDATED_AT_FIELD)
            .set_is_nullable(false)
            .default_is_updated_at()
    }

    pub fn timestamps(&mut self) {
        let mut _created_at = self.created_at();
        let mut _updated_at = self.updated_at();
    }

    pub fn blame(&mut self) {
        let mut _creator = self
            .ulid(CREATOR_FIELD)
            .set_is_nullable(false)
            .references_without_cascade_delete(USER_TABLE, ID_FIELD);

        let mut _editor = self
            .ulid(EDITOR_FIELD)
            .set_is_nullable(true)
            .references_without_cascade_delete(USER_TABLE, ID_FIELD);
    }

    pub fn soft_deletable(&mut self) -> &mut BaseColumn {
        self.timestamp(DELETED_AT_FIELD).set_is_nullable(true)
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

    pub fn index(&mut self, columns: &[&str]) -> &mut Self {
        if self.indexes.is_none() {
            self.indexes = Some(Vec::new());
        }

        if let Some(indexes) = &mut self.indexes {
            indexes.push(IndexType::Index(IndexProp::new(columns, false)));
        }

        self
    }

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
