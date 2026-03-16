#![feature(prelude_import)]
#![allow(dead_code)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::collections::HashSet;
use dirtybase_db::{
    base::manager::Manager, connector::sqlite::make_sqlite_in_memory_manager,
    types::ArcUuid7,
};
use dirtybase_db_macro::DirtyTable;
fn main() {
    let body = async {
        let manager = setup_db().await;
        let p = TopUser {
            id: None,
            list: HashSet::new(),
        };
    };
    let body = {
        if false {
            let _: &dyn ::core::future::Future<Output = ()> = &body;
        }
        body
    };
    #[allow(
        clippy::expect_used,
        clippy::diverging_sub_expression,
        clippy::needless_return,
        clippy::unwrap_in_result
    )]
    {
        use tokio::runtime::Builder;
        return Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
async fn setup_db() -> Manager {
    let manager = make_sqlite_in_memory_manager().await;
    create_tables(&manager).await;
    manager
}
async fn create_tables(_manager: &Manager) {}
struct TopUser {
    id: Option<ArcUuid7>,
    list: HashSet<i32>,
}
#[automatically_derived]
impl ::core::fmt::Debug for TopUser {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "TopUser",
            "id",
            &self.id,
            "list",
            &&self.list,
        )
    }
}
#[automatically_derived]
impl ::core::default::Default for TopUser {
    #[inline]
    fn default() -> TopUser {
        TopUser {
            id: ::core::default::Default::default(),
            list: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
impl ::core::clone::Clone for TopUser {
    #[inline]
    fn clone(&self) -> TopUser {
        TopUser {
            id: ::core::clone::Clone::clone(&self.id),
            list: ::core::clone::Clone::clone(&self.list),
        }
    }
}
impl TopUser {
    pub fn from_column_for_id<'a>(
        field: Option<&'a ::dirtybase_common::db::field_values::FieldValue>,
    ) -> Option<ArcUuid7> {
        ::dirtybase_common::db::field_values::FieldValue::from_ref_option_into_option(
            field,
        )
    }
    pub fn from_column_for_list<'a>(
        field: Option<&'a ::dirtybase_common::db::field_values::FieldValue>,
    ) -> HashSet<i32> {
        ::dirtybase_common::db::field_values::FieldValue::from_ref_option_into(field)
    }
    pub fn into_column_for_id(
        &self,
    ) -> Option<::dirtybase_common::db::field_values::FieldValue> {
        if let Some(value) = &self.id { Some(value.clone().into()) } else { None }
    }
    pub fn into_column_for_list(
        &self,
    ) -> Option<::dirtybase_common::db::field_values::FieldValue> {
        Some(self.list.clone().into())
    }
    pub fn col_name_for_id() -> &'static str {
        "id"
    }
    pub fn col_name_for_list() -> &'static str {
        "list"
    }
    pub fn from_struct_column_value(
        cv: &::dirtybase_common::db::types::StructuredColumnAndValue,
        key: Option<&str>,
    ) -> Option<Self> {
        if let Some(name) = key {
            if let Some(values) = cv.get(name) {
                Some(values.clone().into())
            } else {
                None
            }
        } else {
            ::dirtybase_common::db::types::FromColumnAndValue::from_column_value(
                    cv.clone().fields(),
                )
                .ok()
        }
    }
    pub fn hash_from_struct_column_value(
        cv: &::dirtybase_common::db::types::StructuredColumnAndValue,
        key: Option<&str>,
    ) -> Option<u64> {
        if let Some(name) = key {
            if let Some(::dirtybase_common::db::field_values::FieldValue::Object(v)) = cv
                .get(name)
            {
                if let Some(value) = v
                    .get(<Self as ::dirtybase_common::db::TableModel>::id_column())
                {
                    let id_value = Self::from_column_for_id(Some(value));
                    return Some(
                        <Self as ::dirtybase_common::db::TableModel>::hash_from_id_value(
                            &id_value,
                        ),
                    );
                }
            }
        } else {
            if let Some(value) = cv
                .get(<Self as ::dirtybase_common::db::TableModel>::id_column())
            {
                let id_value = Self::from_column_for_id(Some(value));
                return Some(
                    <Self as ::dirtybase_common::db::TableModel>::hash_from_id_value(
                        &id_value,
                    ),
                );
            }
        }
        None
    }
    pub fn into_embeddable(&self) -> ::dirtybase_common::db::field_values::FieldValue {
        ::dirtybase_common::db::field_values::FieldValue::from(self)
    }
    pub fn repo_instance(
        db_manager: &::dirtybase_common::db::base::manager::Manager,
    ) -> TopUserRepo {
        TopUserRepo::new(db_manager)
    }
}
impl ::dirtybase_common::db::TableModel for TopUser {
    fn entity_hash(&self) -> u64 {
        let mut s = ::std::hash::DefaultHasher::new();
        ::std::hash::Hash::hash(&self.id, &mut s);
        ::std::hash::Hasher::finish(&s)
    }
    fn table_name() -> &'static str {
        "top_users"
    }
    fn foreign_id_column() -> &'static str {
        "top_user_id"
    }
    fn created_at_column() -> Option<&'static str> {
        None
    }
    fn updated_at_column() -> Option<&'static str> {
        None
    }
    fn deleted_at_column() -> Option<&'static str> {
        None
    }
    fn table_columns() -> Vec<&'static str> {
        let main = <[_]>::into_vec(::alloc::boxed::box_new(["id", "list"]));
        [main].concat()
    }
}
impl ::dirtybase_common::db::types::FromColumnAndValue for TopUser {
    fn from_column_value(
        cv: ::dirtybase_common::db::types::ColumnAndValue,
    ) -> Result<Self, ::dirtybase_common::anyhow::Error> {
        Ok(Self {
            id: Self::from_column_for_id(
                if cv.contains_key("id") {
                    cv.get("id")
                } else {
                    match cv.get("top_users") {
                        Some(
                            ::dirtybase_common::db::field_values::FieldValue::Object(c),
                        ) => c.get("id").clone(),
                        _ => None,
                    }
                },
            ),
            list: Self::from_column_for_list(
                if cv.contains_key("list") {
                    cv.get("list")
                } else {
                    match cv.get("top_users") {
                        Some(
                            ::dirtybase_common::db::field_values::FieldValue::Object(c),
                        ) => c.get("list").clone(),
                        _ => None,
                    }
                },
            ),
        })
    }
}
impl ::dirtybase_common::db::types::ToColumnAndValue for TopUser {
    fn to_column_value(
        &self,
    ) -> Result<
        ::dirtybase_common::db::types::ColumnAndValue,
        ::dirtybase_common::anyhow::Error,
    > {
        Ok(
            ::dirtybase_common::db::ColumnAndValueBuilder::new()
                .try_to_insert_field_value("id", self.into_column_for_id())
                .try_to_insert_field_value("list", self.into_column_for_list())
                .build(),
        )
    }
}
impl ::dirtybase_common::db::types::ToColumnAndValue for &TopUser {
    fn to_column_value(
        &self,
    ) -> Result<
        ::dirtybase_common::db::types::ColumnAndValue,
        ::dirtybase_common::anyhow::Error,
    > {
        Ok(
            ::dirtybase_common::db::ColumnAndValueBuilder::new()
                .try_to_insert_field_value("id", self.into_column_for_id())
                .try_to_insert_field_value("list", self.into_column_for_list())
                .build(),
        )
    }
}
impl From<::dirtybase_common::db::field_values::FieldValue> for TopUser {
    fn from(value: ::dirtybase_common::db::field_values::FieldValue) -> Self {
        let cv = ::dirtybase_common::db::types::ColumnAndValue::from(value);
        if cv.is_empty() {
            Self::default()
        } else {
            ::dirtybase_common::db::types::FromColumnAndValue::from_column_value(cv)
                .expect("could not convert from field value")
        }
    }
}
impl From<&::dirtybase_common::db::field_values::FieldValue> for TopUser {
    fn from(value: &::dirtybase_common::db::field_values::FieldValue) -> Self {
        value.clone().into()
    }
}
/// Repository.
///
/// This struct is autogenerated
pub struct TopUserRepo {
    builder: ::dirtybase_common::db::base::query::QueryBuilder,
    manager: ::dirtybase_common::db::base::manager::Manager,
    include_deleted_rec: bool,
    return_only_deleted_rec: bool,
    relation: ::std::collections::HashMap<
        String,
        ::dirtybase_common::db::repo_relation::Relation<TopUser>,
    >,
}
#[automatically_derived]
impl ::core::fmt::Debug for TopUserRepo {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field5_finish(
            f,
            "TopUserRepo",
            "builder",
            &self.builder,
            "manager",
            &self.manager,
            "include_deleted_rec",
            &self.include_deleted_rec,
            "return_only_deleted_rec",
            &self.return_only_deleted_rec,
            "relation",
            &&self.relation,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for TopUserRepo {
    #[inline]
    fn clone(&self) -> TopUserRepo {
        TopUserRepo {
            builder: ::core::clone::Clone::clone(&self.builder),
            manager: ::core::clone::Clone::clone(&self.manager),
            include_deleted_rec: ::core::clone::Clone::clone(&self.include_deleted_rec),
            return_only_deleted_rec: ::core::clone::Clone::clone(
                &self.return_only_deleted_rec,
            ),
            relation: ::core::clone::Clone::clone(&self.relation),
        }
    }
}
impl TopUserRepo {
    pub fn new(manager: &::dirtybase_common::db::base::manager::Manager) -> Self {
        Self {
            builder: ::dirtybase_common::db::base::query::QueryBuilder::new(
                <TopUser as ::dirtybase_common::db::table_model::TableModel>::table_name(),
                ::dirtybase_common::db::base::query::QueryAction::query(),
            ),
            manager: manager.clone(),
            relation: ::std::collections::HashMap::new(),
            include_deleted_rec: false,
            return_only_deleted_rec: false,
        }
    }
    pub async fn cursor_paginate(
        &mut self,
        cursor: Option<::dirtybase_common::db::base::cursor_builder::CursorBuilder>,
    ) -> ::dirtybase_common::db::base::cursor_builder::CursorResult<TopUser> {
        let mut rows_map = Vec::<TopUser>::new();
        let mut join_field_values = ::std::collections::HashMap::new();
        let mut rows_rel_map = ::std::collections::HashMap::new();
        let mut cursor = if let Some(cursor) = cursor {
            cursor
        } else {
            ::dirtybase_common::db::base::cursor_builder::CursorBuilder::new(
                &<TopUser as ::dirtybase_common::db::table_model::TableModel>::id_column(),
                None,
            )
        };
        self.builder
            .select_multiple(
                &<TopUser as ::dirtybase_common::db::table_model::TableModel>::table_query_col_aliases(
                    None,
                ),
            );
        let cursor_result = self
            .manager
            .execute_query(self.builder.clone())
            .cursor_paginate(cursor)
            .await;
        let (cursor, result) = cursor_result.parts();
        match result {
            Ok(mut raw_list) => {
                for row in raw_list {
                    if let Some(row_entity) = TopUser::from_struct_column_value(
                        &row,
                        Some(
                            <TopUser as ::dirtybase_common::db::table_model::TableModel>::table_name(),
                        ),
                    ) {
                        rows_map.push(row_entity);
                    }
                }
                for (name, rel) in &self.relation {
                    if let Err(e) = rel
                        .clone()
                        .process(
                            &name,
                            &self.manager,
                            &rows_map,
                            &mut join_field_values,
                            &mut rows_rel_map,
                        )
                        .await
                    {
                        *self = Self::new(&self.manager);
                        return ::dirtybase_common::db::base::cursor_builder::CursorResult::<
                            TopUser,
                        >::new(cursor, Err(e));
                    }
                }
                for row_entity in &mut rows_map {
                    let row_hash = ::dirtybase_common::db::table_model::TableModel::entity_hash(
                        row_entity,
                    );
                }
                *self = Self::new(&self.manager);
                ::dirtybase_common::db::base::cursor_builder::CursorResult::<
                    TopUser,
                >::new(cursor, Ok(rows_map))
            }
            Err(e) => {
                *self = Self::new(&self.manager);
                ::dirtybase_common::db::base::cursor_builder::CursorResult::<
                    TopUser,
                >::new(cursor, Err(e))
            }
        }
    }
    pub async fn get(
        &mut self,
    ) -> Result<Vec<TopUser>, ::dirtybase_common::anyhow::Error> {
        let mut rows_map = Vec::<TopUser>::new();
        let mut join_field_values = ::std::collections::HashMap::new();
        let mut rows_rel_map = ::std::collections::HashMap::new();
        self.builder
            .select_multiple(
                &<TopUser as ::dirtybase_common::db::table_model::TableModel>::table_query_col_aliases(
                    None,
                ),
            );
        let result = self.manager.execute_query(self.builder.clone()).all().await;
        match result {
            Ok(mut raw_list) => {
                for row in raw_list {
                    if let Some(row_entity) = TopUser::from_struct_column_value(
                        &row,
                        Some(
                            <TopUser as ::dirtybase_common::db::table_model::TableModel>::table_name(),
                        ),
                    ) {
                        let row_hash = ::dirtybase_common::db::table_model::TableModel::entity_hash(
                            &row_entity,
                        );
                        rows_map.push(row_entity);
                    }
                }
                for (name, rel) in &self.relation {
                    if let Err(e) = rel
                        .clone()
                        .process(
                            &name,
                            &self.manager,
                            &rows_map,
                            &mut join_field_values,
                            &mut rows_rel_map,
                        )
                        .await
                    {
                        *self = Self::new(&self.manager);
                        return Err(e);
                    }
                }
                for row_entity in &mut rows_map {
                    let row_hash = ::dirtybase_common::db::table_model::TableModel::entity_hash(
                        row_entity,
                    );
                }
                *self = Self::new(&self.manager);
                Ok(rows_map)
            }
            Err(e) => {
                *self = Self::new(&self.manager);
                Err(e)
            }
        }
    }
    pub async fn one(
        &mut self,
    ) -> Result<Option<TopUser>, ::dirtybase_common::anyhow::Error> {
        match self.limit(1).get().await {
            Ok(mut list) => Ok(list.pop()),
            Err(e) => Err(e),
            _ => Ok(None),
        }
    }
    pub fn limit(&mut self, limit: usize) -> &mut Self {
        self.builder.limit(limit);
        self
    }
    pub async fn latest(
        &mut self,
    ) -> Result<Option<TopUser>, ::dirtybase_common::anyhow::Error> {
        self.builder
            .desc(
                <TopUser as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                    <TopUser as ::dirtybase_common::db::table_model::TableModel>::id_column(),
                ),
            );
        self.one().await
    }
    pub async fn oldest(
        &mut self,
    ) -> Result<Option<TopUser>, ::dirtybase_common::anyhow::Error> {
        self.builder
            .asc(
                <TopUser as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                    <TopUser as ::dirtybase_common::db::table_model::TableModel>::id_column(),
                ),
            );
        self.one().await
    }
    pub async fn count(&mut self) -> Result<i64, ::dirtybase_common::anyhow::Error> {
        let id_column = <TopUser as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
            <TopUser as ::dirtybase_common::db::table_model::TableModel>::id_column(),
        );
        self.builder.count_as(&id_column, "_count_all");
        let result = self.manager.execute_query(self.builder.clone()).fetch_one().await;
        *self = Self::new(&self.manager);
        if let Ok(row) = result {
            match row {
                Some(r) => {
                    let count = if let Some(v) = r.get("_count_all") {
                        ::std::primitive::i64::from(v)
                    } else {
                        0
                    };
                    Ok(count)
                }
                None => Ok(0),
            }
        } else {
            Err(result.err().expect("could not run 'count' query"))
        }
    }
    pub fn filter(
        &mut self,
        mut callback: impl FnOnce(&mut ::dirtybase_common::db::base::query::QueryBuilder),
    ) -> &mut Self {
        callback(&mut self.builder);
        self
    }
    pub async fn by_id(
        &mut self,
        id: ArcUuid7,
    ) -> Result<Option<TopUser>, ::dirtybase_common::anyhow::Error> {
        self.builder
            .is_eq(
                <TopUser as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                    <TopUser as ::dirtybase_common::db::table_model::TableModel>::id_column(),
                ),
                id,
            );
        self.one().await
    }
    pub async fn id_in(
        &mut self,
        ids: Vec<ArcUuid7>,
    ) -> Result<Vec<TopUser>, ::dirtybase_common::anyhow::Error> {
        self.builder
            .is_in(
                <TopUser as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                    <TopUser as ::dirtybase_common::db::table_model::TableModel>::id_column(),
                ),
                ids,
            );
        self.get().await
    }
    pub fn table_name() -> &'static str {
        <TopUser as ::dirtybase_common::db::table_model::TableModel>::table_name()
    }
    pub fn is_soft_deletable() -> bool {
        <TopUser as ::dirtybase_common::db::table_model::TableModel>::deleted_at_column()
            .is_some()
    }
    pub fn is_timestampable() -> bool {
        <TopUser as ::dirtybase_common::db::table_model::TableModel>::created_at_column()
            .is_some()
            && <TopUser as ::dirtybase_common::db::table_model::TableModel>::created_at_column()
                .is_some()
    }
    pub async fn insert(
        &mut self,
        mut record: TopUser,
    ) -> Result<TopUser, ::dirtybase_common::anyhow::Error> {
        let result = self.manager.insert_into::<TopUser>(record).await?;
        if let Some(record) = result.record().cloned() {
            return <TopUser as ::dirtybase_common::db::types::FromColumnAndValue>::from_column_value(
                record,
            );
        } else {
            self.builder
                .is_eq(
                    <TopUser as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                        <TopUser as ::dirtybase_common::db::table_model::TableModel>::id_column(),
                    ),
                    result.last_insert_id(),
                );
            return self
                .one()
                .await?
                .ok_or(
                    ::anyhow::__private::must_use({
                        let error = ::anyhow::__private::format_err(
                            format_args!("could not get back inserted record"),
                        );
                        error
                    }),
                );
        }
    }
    pub async fn update(
        &mut self,
        mut record: TopUser,
    ) -> Result<TopUser, ::dirtybase_common::anyhow::Error> {
        let id = record.id.clone().expect("expected a 'Some' ID but found 'None'");
        _ = self
            .manager
            .update_table::<
                TopUser,
            >(
                record,
                |qb| {
                    qb.is_eq(
                        <TopUser as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                            <TopUser as ::dirtybase_common::db::table_model::TableModel>::id_column(),
                        ),
                        id.clone(),
                    );
                },
            )
            .await?;
        match self.by_id(id).await? {
            Some(v) => Ok(v),
            None => {
                Err(
                    ::anyhow::__private::must_use({
                        let error = ::anyhow::__private::format_err(
                            format_args!("could not retrieve updated model"),
                        );
                        error
                    }),
                )
            }
        }
    }
    pub async fn delete(
        &mut self,
        mut record: TopUser,
    ) -> Result<TopUser, ::dirtybase_common::anyhow::Error> {
        let id = record.id.clone().expect("expected a 'Some' ID but found 'None'");
        _ = self.delete_by_id(id).await?;
        Ok(record)
    }
    pub async fn delete_by_id(
        &mut self,
        id: ArcUuid7,
    ) -> Result<(), ::dirtybase_common::anyhow::Error> {
        _ = self
            .manager
            .delete_from_table::<
                TopUser,
            >(|qb| {
                qb.is_eq(
                    <TopUser as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                        <TopUser as ::dirtybase_common::db::table_model::TableModel>::id_column(),
                    ),
                    id,
                );
            })
            .await?;
        Ok(())
    }
    pub async fn destroy(
        &mut self,
        record: TopUser,
    ) -> Result<(), ::dirtybase_common::anyhow::Error> {
        let id = record.id.clone().expect("expected a 'Some' ID but found 'None'");
        self.destroy_by_id(id).await
    }
    pub async fn destroy_by_id(
        &mut self,
        id: ArcUuid7,
    ) -> Result<(), ::dirtybase_common::anyhow::Error> {
        self.manager
            .delete_from_table::<
                TopUser,
            >(|qb| {
                qb.is_eq(
                    <TopUser as ::dirtybase_common::db::table_model::TableModel>::prefix_with_tbl(
                        <TopUser as ::dirtybase_common::db::table_model::TableModel>::id_column(),
                    ),
                    id,
                );
            })
            .await
    }
    pub fn col_id() -> &'static str {
        "top_users.id"
    }
    pub fn col_list() -> &'static str {
        "top_users.list"
    }
}
