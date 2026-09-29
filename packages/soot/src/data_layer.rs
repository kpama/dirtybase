use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use dirtybase_db::{
    base::{
        manager::Manager,
        query::{QueryBuilder, WhereJoin},
        query_operators::Operator,
    },
    field_values::FieldValue,
    types::ColumnAndValue,
};

use crate::{
    aggregate::{Aggregate, AggregateKind},
    attribute::is_nil,
    changeset::{Changeset, ChangesetKind},
    domain::Domain,
    error::{Error, ErrorList, Result, from_db},
    query::{Filter, FilterJoin, FilterOperator, Query, SortDirection},
    record::Record,
    relationship::{LoadedRelationship, RelationshipType},
    resource::ResourceDef,
};

/// Where a resource's records actually live, and how they are loaded.
///
/// This is Ash's data layer. Everything above it is written once against
/// metadata; swapping this trait swaps storage. The soot implementation
/// (`RelationalDataLayer`) puts records in a dirtybase relational database, but
/// nothing in the engine assumes that beyond this trait.
#[async_trait]
pub trait DataLayer: Send + Sync {
    /// Insert a record and return it as stored.
    async fn create(&self, resource: Arc<ResourceDef>, changeset: &Changeset) -> Result<Record>;

    /// Apply a partial update and return the record as stored.
    async fn update(&self, resource: Arc<ResourceDef>, changeset: &Changeset) -> Result<Record>;

    /// Remove a record and return it as it was.
    async fn destroy(&self, resource: Arc<ResourceDef>, changeset: &Changeset) -> Result<Record>;

    async fn read(&self, resource: Arc<ResourceDef>, query: &Query) -> Result<Vec<Record>>;

    async fn read_one(&self, resource: Arc<ResourceDef>, query: &Query) -> Result<Option<Record>>;

    /// Load the named relationships onto the given records.
    async fn load(
        &self,
        resource: Arc<ResourceDef>,
        records: &mut [Record],
        relationships: &[String],
    ) -> Result<()>;

    /// Total matching records, ignoring limit and offset.
    async fn count(&self, resource: Arc<ResourceDef>, query: &Query) -> Result<i64>;

    /// Compute an aggregate over the query's result set.
    async fn aggregate(
        &self,
        resource: Arc<ResourceDef>,
        query: &Query,
        aggregate: &Aggregate,
    ) -> Result<Option<FieldValue>>;

    /// Whether the backing store can be written to.
    fn is_writable(&self) -> bool;
}

/// A data layer over a dirtybase relational database.
///
/// Reads go through `QueryBuilder` so filters, sorting and paging are rendered
/// by dirtybase rather than reassembled here, and writes go through `Manager`
/// so the write-stickiness and cache invalidation events still fire.
///
/// The domain is held so relationships can be followed: loading a `has_many`
/// means looking up the destination resource's declaration, which only the
/// domain knows about.
pub struct RelationalDataLayer {
    manager: Manager,
    domain: Arc<Domain>,
}

impl std::fmt::Debug for RelationalDataLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RelationalDataLayer({:?})", self.manager.db_kind())
    }
}

impl RelationalDataLayer {
    /// A layer with an empty domain. Followed relationships will not resolve
    /// unless the resource's destinations are registered later.
    pub fn new(manager: Manager) -> Self {
        Self {
            manager,
            domain: Arc::new(Domain::new()),
        }
    }

    pub fn with_domain(manager: Manager, domain: Arc<Domain>) -> Self {
        Self { manager, domain }
    }

    pub fn manager(&self) -> &Manager {
        &self.manager
    }

    pub fn domain(&self) -> &Arc<Domain> {
        &self.domain
    }

    /// Render a query against a resource into a `QueryBuilder`.
    ///
    /// This is the whole translation layer between soot's metadata-driven query
    /// and dirtybase's SQL builder: attribute names become column names, and
    /// every filter becomes an `Operator`.
    fn build(&self, resource: &ResourceDef, query: &Query) -> QueryBuilder {
        let mut builder = self.manager.table(resource.table_name());

        // Soft deleted rows are invisible unless explicitly asked for.
        if resource.is_soft_deletable() && !query.has_flag("include_deleted") {
            builder.without_trashed();
        }

        for filter in query.filters() {
            apply_filter(&mut builder, resource, filter);
        }

        for (attribute, direction) in query.sorting() {
            let column = column_for(resource, attribute);
            match direction {
                SortDirection::Ascending => builder.asc(column),
                SortDirection::Descending => builder.desc(column),
            };
        }

        if let Some(limit) = query.limit_by() {
            builder.limit(limit);
        }
        if let Some(offset) = query.offset_by() {
            builder.offset(offset);
        }

        builder
    }

    /// Read every matching row and turn it into a `Record`, applying the
    /// declared attribute types so callers see the types they declared rather
    /// than whatever the driver produced.
    ///
    /// Columns are read in full and pruned afterwards. Restricting the column
    /// list in SQL would save some transfer, but it would also make every
    /// relationship load have to know which columns the caller asked for, and
    /// getting that wrong turns into a missing value rather than an error.
    async fn read_records(
        &self,
        resource: &Arc<ResourceDef>,
        query: &Query,
    ) -> Result<Vec<Record>> {
        let mut records = self.read_rows(resource, query).await?;
        self.finish(resource, query, &mut records).await?;
        Ok(records)
    }

    /// The records as stored, with nothing derived added.
    ///
    /// [`RelationalDataLayer::finish`] layers calculations, aggregates and
    /// loads on top. Anything that only needs the stored columns — the key
    /// collection inside a relationship aggregate, most importantly — uses this
    /// directly, since finishing there would recurse back into the aggregate
    /// that asked for it.
    async fn read_rows(&self, resource: &Arc<ResourceDef>, query: &Query) -> Result<Vec<Record>> {
        let builder = self.build(resource, query);
        let rows = self
            .manager
            .execute_query(builder)
            .fetch_all()
            .await
            .map_err(|e| from_db("read failed", e))?;

        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            let mut record = hydrate(resource, row.fields());
            if !query.selects().is_empty() {
                let mut keep = std::collections::BTreeSet::new();
                keep.insert(resource.primary_key_column().to_string());
                for attribute in query.selects() {
                    keep.insert(
                        resource
                            .find_attribute(attribute)
                            .map(|a| a.column_name().to_string())
                            .unwrap_or_else(|| attribute.clone()),
                    );
                }
                record.select_columns(&keep);
            }
            records.push(record);
        }
        Ok(records)
    }

    /// Complete a freshly read set of records: run the calculations, compute the
    /// aggregates, and follow the relationships the query asked for.
    ///
    /// A read that skipped this would hand back records missing exactly the
    /// fields the caller requested, which is far harder to notice than a query
    /// that failed outright.
    async fn finish(
        &self,
        resource: &Arc<ResourceDef>,
        query: &Query,
        records: &mut [Record],
    ) -> Result<()> {
        for record in records.iter_mut() {
            crate::domain::apply_selected_calculations(resource, query.calculates(), record).await;
        }

        for aggregate in query.aggregates() {
            if let Some(value) = self
                .aggregate(Arc::clone(resource), query, aggregate)
                .await?
            {
                // An aggregate is a property of the whole result set, so it is
                // the same on every record.
                for record in records.iter_mut() {
                    record.put_aggregated(aggregate.name(), value.clone());
                }
            }
        }

        let loads: Vec<String> = query.loads().iter().cloned().collect();
        if !loads.is_empty() {
            self.load(Arc::clone(resource), records, &loads).await?;
        }

        Ok(())
    }

    /// Re-read a single row by primary key, which is how a write action returns
    /// the record in its final state rather than its pre-write state.
    async fn read_back(
        &self,
        resource: &Arc<ResourceDef>,
        primary_key: &FieldValue,
    ) -> Result<Record> {
        let column = resource.primary_key_column();
        let table = resource.table_name();
        let value = primary_key.clone();

        let row = self
            .manager
            .select_from_table(table, move |builder| {
                builder.is_eq(column, value);
            })
            .fetch_one()
            .await
            .map_err(|e| from_db("write-back read failed", e))?;

        match row {
            Some(row) => Ok(hydrate(resource, row.fields())),
            None => Err(Error::framework(format!(
                "`{}` was written but could not be read back",
                resource.name()
            ))
            .into()),
        }
    }

    /// The columns to write, with timestamps and soft delete maintained.
    ///
    /// Doing this here rather than in a change is deliberate: it applies to
    /// every write path, including ones a caller writes by hand, so
    /// `created_at` cannot be forgotten.
    fn write_columns(&self, resource: &ResourceDef, changeset: &Changeset) -> ColumnAndValue {
        let mut columns = changeset.to_column_and_value();

        // Fill in the primary key when the caller did not supply one, so a
        // create works without the caller having to know the id scheme.
        let primary_key = resource.primary_key_column();
        if !columns.contains_key(primary_key) {
            if let Some(attribute) = resource.primary_key_attribute().ok()
                && let Some(value) = attribute.starting_value()
            {
                columns.insert(primary_key.to_string(), value);
            }
        }

        let now = FieldValue::DateTime(chrono::Utc::now());
        match changeset.kind() {
            ChangesetKind::Create => {
                if resource.has_timestamps() {
                    // A newly inserted record has both stamps set, not just
                    // `created_at`. Leaving `updated_at` nil would mean "never
                    // updated", which is wrong for a row that is being created
                    // now, and it reads back as NULL.
                    columns
                        .entry(resource.created_at_column().to_string())
                        .or_insert_with(|| now.clone());
                    columns
                        .entry(resource.updated_at_column().to_string())
                        .or_insert_with(|| now.clone());
                }
            }
            ChangesetKind::Update | ChangesetKind::Destroy => {
                if resource.has_timestamps() {
                    columns.insert(resource.updated_at_column().to_string(), now);
                }
            }
        }

        columns
    }
}

#[async_trait]
impl DataLayer for RelationalDataLayer {
    async fn create(&self, resource: Arc<ResourceDef>, changeset: &Changeset) -> Result<Record> {
        let columns = self.write_columns(&resource, changeset);
        let primary_key_value = columns
            .get(resource.primary_key_column())
            .cloned()
            .ok_or_else(|| {
                Error::no_primary_key(format!(
                    "`{}` has no primary key value to insert",
                    resource.name()
                ))
            })?;

        let result = self
            .manager
            .insert(resource.table_name(), columns)
            .await
            .map_err(|e| from_db("insert failed", e))?;

        // Prefer what the driver echoed back, then fall back to a re-read.
        if let Some(record) = result.record() {
            let mut record = hydrate(&resource, record.clone());
            if record.primary_key(resource.primary_key_column()).is_none() {
                record.set(resource.primary_key_column(), primary_key_value);
            }
            return Ok(record);
        }

        self.read_back(&resource, &primary_key_value).await
    }

    async fn update(&self, resource: Arc<ResourceDef>, changeset: &Changeset) -> Result<Record> {
        let primary_key_value = changeset.primary_key().ok_or_else(|| {
            Error::no_primary_key(format!(
                "cannot update `{}` without a primary key",
                resource.name()
            ))
        })?;

        let mut columns = self.write_columns(&resource, changeset);
        // The primary key is the target of the update, not something to write.
        columns.remove(resource.primary_key_column());

        if columns.is_empty() {
            return self.read_back(&resource, &primary_key_value).await;
        }

        let table = resource.table_name();
        let column = resource.primary_key_column();
        let value = primary_key_value.clone();

        self.manager
            .update(table, columns, move |builder| {
                builder.is_eq(column, value);
            })
            .await
            .map_err(|e| from_db("update failed", e))?;

        self.read_back(&resource, &primary_key_value).await
    }

    async fn destroy(&self, resource: Arc<ResourceDef>, changeset: &Changeset) -> Result<Record> {
        let record = changeset.data().clone();
        let primary_key_value = changeset.primary_key().ok_or_else(|| {
            Error::no_primary_key(format!(
                "cannot destroy `{}` without a primary key",
                resource.name()
            ))
        })?;

        let table = resource.table_name();
        let column = resource.primary_key_column();
        let value = primary_key_value;

        if resource.is_soft_deletable() {
            // Soft delete: stamp the column rather than removing the row, so the
            // record stays available to an audit trail.
            let mut columns = ColumnAndValue::new();
            columns.insert(
                resource.deleted_at_column().to_string(),
                FieldValue::DateTime(chrono::Utc::now()),
            );
            if resource.has_timestamps() {
                columns.insert(
                    resource.updated_at_column().to_string(),
                    FieldValue::DateTime(chrono::Utc::now()),
                );
            }
            let target = value.clone();
            self.manager
                .update(table, columns, move |builder| {
                    builder.is_eq(column, target);
                })
                .await
                .map_err(|e| from_db("soft delete failed", e))?;
            return Ok(record);
        }

        self.manager
            .delete(table, move |builder| {
                builder.is_eq(column, value);
            })
            .await
            .map_err(|e| from_db("delete failed", e))?;

        Ok(record)
    }

    async fn read(&self, resource: Arc<ResourceDef>, query: &Query) -> Result<Vec<Record>> {
        self.read_records(&resource, query).await
    }

    async fn read_one(&self, resource: Arc<ResourceDef>, query: &Query) -> Result<Option<Record>> {
        Ok(self
            .read_records(&resource, query)
            .await?
            .into_iter()
            .next())
    }

    async fn count(&self, resource: Arc<ResourceDef>, query: &Query) -> Result<i64> {
        let mut builder = self.build(&resource, query);
        builder.count_as(resource.primary_key_column(), "soot_count");

        let row = self
            .manager
            .execute_query(builder)
            .fetch_one()
            .await
            .map_err(|e| from_db("count failed", e))?;

        Ok(match row {
            Some(row) => match row.get("soot_count") {
                Some(FieldValue::I64(count)) => *count,
                Some(FieldValue::F64(count)) => *count as i64,
                _ => 0,
            },
            None => 0,
        })
    }

    async fn aggregate(
        &self,
        resource: Arc<ResourceDef>,
        query: &Query,
        aggregate: &Aggregate,
    ) -> Result<Option<FieldValue>> {
        match aggregate.relationship() {
            // A relationship aggregate is a second query over the destination
            // resource, filtered to the ids of the records being read.
            Some(relationship_name) => {
                let Some(relationship) = resource.find_relationship(relationship_name) else {
                    return Err(Error::unknown_field(
                        relationship_name,
                        format!(
                            "`{relationship_name}` is not a relationship of `{}`",
                            resource.name()
                        ),
                    )
                    .into());
                };
                // Deliberately the raw read: finishing these records would
                // recompute the very aggregate being computed.
                let records = self.read_rows(&resource, query).await?;
                let mut keys: Vec<FieldValue> = Vec::new();
                for record in &records {
                    if let Some(key) = relationship.join_key(record)? {
                        keys.push(key);
                    }
                }

                let destination_table = self
                    .destination_definition(relationship.destination())?
                    .table_name()
                    .to_string();
                let destination_column = relationship.destination_attribute().to_string();
                let keys_for_query = keys.clone();
                let filter_column = destination_column.clone();
                let empty = keys_for_query.is_empty();

                let row = self
                    .manager
                    .select_from_table(&destination_table, move |builder| {
                        if empty {
                            // No source records, so the aggregate covers nothing.
                            // A filter on a non-existent column is not portable,
                            // so constrain by a value that can never match.
                            builder.is_eq("soot_never", FieldValue::Null);
                        } else {
                            builder.is_in(&filter_column, keys_for_query);
                        }
                        add_aggregate(builder, &aggregate);
                    })
                    .fetch_one()
                    .await
                    .map_err(|e| from_db("aggregate failed", e))?;

                Ok(row.and_then(|row| row.get("soot_aggregate").cloned()))
            }
            // An aggregate over the records themselves pushes the function down
            // into SQL so it does not have to materialise the whole result set.
            None => {
                let table = resource.table_name().to_string();
                let aggregate = aggregate.clone();
                let row = self
                    .manager
                    .select_from_table(&table, move |builder| {
                        add_aggregate(builder, &aggregate);
                    })
                    .fetch_one()
                    .await
                    .map_err(|e| from_db("aggregate failed", e))?;

                Ok(row.and_then(|row| row.get("soot_aggregate").cloned()))
            }
        }
    }

    async fn load(
        &self,
        resource: Arc<ResourceDef>,
        records: &mut [Record],
        relationships: &[String],
    ) -> Result<()> {
        for name in relationships {
            let Some(relationship) = resource.find_relationship(name) else {
                return Err(Error::unknown_field(
                    name,
                    format!("`{name}` is not a relationship of `{}`", resource.name()),
                )
                .into());
            };

            // A relationship may bring its own loading logic, in which case the
            // data layer only has to hand over each source record.
            if let Some(loader) = relationship.custom_loader() {
                for record in records.iter_mut() {
                    let loaded = loader(record.clone()).await?;
                    record.put_loaded(name, loaded);
                }
                continue;
            }

            let destination = self.destination_definition(relationship.destination())?;
            let values = self
                .load_relationship(relationship, &destination, records)
                .await?;
            for (index, record) in records.iter_mut().enumerate() {
                let loaded =
                    values
                        .get(index)
                        .cloned()
                        .unwrap_or(match relationship.relationship_type() {
                            RelationshipType::HasMany | RelationshipType::ManyToMany => {
                                LoadedRelationship::ToMany(Vec::new())
                            }
                            _ => LoadedRelationship::ToOne(None),
                        });
                record.put_loaded(name, loaded);
            }
        }
        Ok(())
    }

    fn is_writable(&self) -> bool {
        self.manager.is_writable()
    }
}

impl RelationalDataLayer {
    /// Resolve a relationship's destination resource declaration through the
    /// domain, so a relationship can be followed without the caller having to
    /// thread resource knowledge in by hand.
    fn destination_definition(&self, destination: &str) -> Result<Arc<ResourceDef>> {
        self.domain.resource(destination).map(Arc::clone)
    }

    /// Load one relationship for a batch of source records, returning one
    /// `LoadedRelationship` per source record in the same order.
    async fn load_relationship(
        &self,
        relationship: &crate::relationship::Relationship,
        destination: &Arc<ResourceDef>,
        records: &[Record],
    ) -> Result<Vec<LoadedRelationship>> {
        let ty = relationship.relationship_type();
        let destination_column = relationship.destination_attribute();

        if ty == RelationshipType::ManyToMany {
            return self
                .load_many_to_many(relationship, destination, records)
                .await;
        }

        if ty.joins_from_source() {
            // belongs_to: one lookup per distinct foreign key.
            let mut out = vec![LoadedRelationship::ToOne(None); records.len()];
            let mut seen = std::collections::BTreeSet::new();
            let mut wanted: Vec<FieldValue> = Vec::new();

            for record in records {
                let Some(key) = relationship.join_key(record)? else {
                    continue;
                };
                if seen.insert(key.to_string()) {
                    wanted.push(key);
                }
            }

            if wanted.is_empty() {
                return Ok(out);
            }

            let table = destination.table_name().to_string();
            let column = destination
                .find_attribute(destination_column)
                .map(|attribute| attribute.column_name().to_string())
                .unwrap_or_else(|| destination_column.to_string());
            let keys = wanted.clone();
            let rows = self
                .manager
                .select_from_table(&table, move |builder| {
                    builder.is_in(&column, keys);
                })
                .fetch_all()
                .await
                .map_err(|e| from_db("belongs_to load failed", e))?;

            let mut by_key: BTreeMap<String, Record> = BTreeMap::new();
            for row in rows {
                let record = hydrate(destination, row.fields());
                if let Some(key) = record.primary_key(destination.primary_key_column()) {
                    by_key.insert(key.to_string(), record);
                }
            }

            for (index, record) in records.iter().enumerate() {
                if let Some(key) = relationship.join_key(record)?
                    && let Some(found) = by_key.get(&key.to_string())
                {
                    out[index] = LoadedRelationship::ToOne(Some(found.clone()));
                }
            }
            return Ok(out);
        }

        // has_one / has_many: fetch every matching destination row, then bucket
        // them by the foreign key value.
        let keys: Vec<FieldValue> = records
            .iter()
            .filter_map(|record| {
                record
                    .primary_key(relationship.source_attribute())
                    .or_else(|| record.get(relationship.source_attribute()))
            })
            .collect();

        if keys.is_empty() {
            return Ok(vec![LoadedRelationship::ToMany(Vec::new()); records.len()]);
        }

        let table = destination.table_name().to_string();
        let column = destination
            .find_attribute(destination_column)
            .map(|attribute| attribute.column_name().to_string())
            .unwrap_or_else(|| destination_column.to_string());
        let wanted = keys.clone();
        let soft_delete_column = destination.deleted_at_column().to_string();
        let soft_deletable = destination.is_soft_deletable();

        let rows = self
            .manager
            .select_from_table(&table, move |builder| {
                builder.is_in(&column, wanted);
                if soft_deletable {
                    builder.is_null(&soft_delete_column);
                }
            })
            .fetch_all()
            .await
            .map_err(|e| from_db("has_many load failed", e))?;

        let mut by_key: BTreeMap<String, Vec<Record>> = BTreeMap::new();
        for row in rows {
            let record = hydrate(destination, row.fields());
            if let Some(key) = record.get(destination_column) {
                by_key.entry(key.to_string()).or_default().push(record);
            }
        }

        let mut out = Vec::with_capacity(records.len());
        for record in records {
            let key = record
                .primary_key(relationship.source_attribute())
                .or_else(|| record.get(relationship.source_attribute()))
                .map(|value| value.to_string())
                .unwrap_or_default();
            let found = by_key.get(&key).cloned().unwrap_or_default();
            out.push(if ty == RelationshipType::HasOne {
                LoadedRelationship::ToOne(found.into_iter().next())
            } else {
                LoadedRelationship::ToMany(found)
            });
        }
        Ok(out)
    }

    /// Many-to-many: walk the join resource, then resolve the destination rows.
    async fn load_many_to_many(
        &self,
        relationship: &crate::relationship::Relationship,
        destination: &Arc<ResourceDef>,
        records: &[Record],
    ) -> Result<Vec<LoadedRelationship>> {
        let join_table = relationship
            .join_resource()
            .ok_or_else(|| {
                Error::changeset(format!(
                    "relationship `{}` is many_to_many but names no join resource",
                    relationship.name()
                ))
            })?
            .to_string();
        let this_side = relationship.source_attribute().to_string();
        let other_side = relationship.destination_attribute().to_string();

        let keys: Vec<FieldValue> = records
            .iter()
            .filter_map(|record| {
                record
                    .primary_key(relationship.source_attribute())
                    .or_else(|| record.get(relationship.source_attribute()))
            })
            .collect();

        if keys.is_empty() {
            return Ok(vec![LoadedRelationship::ToMany(Vec::new()); records.len()]);
        }

        let wanted = keys.clone();
        let join_filter = this_side.clone();
        let join = self
            .manager
            .select_from_table(&join_table, move |builder| {
                builder.is_in(&join_filter, wanted);
            })
            .fetch_all()
            .await
            .map_err(|e| from_db("many_to_many join read failed", e))?;

        // Map each source id to the destination ids it links to.
        let mut links: BTreeMap<String, Vec<FieldValue>> = BTreeMap::new();
        let mut destination_ids: Vec<FieldValue> = Vec::new();
        for row in join {
            let values = row.fields();
            let source = values.get(&this_side).cloned();
            let target = values.get(&other_side).cloned();
            if let (Some(source), Some(target)) = (source, target.clone()) {
                links
                    .entry(source.to_string())
                    .or_default()
                    .push(target.clone());
                destination_ids.push(target);
            }
        }

        if destination_ids.is_empty() {
            return Ok(vec![LoadedRelationship::ToMany(Vec::new()); records.len()]);
        }

        let table = destination.table_name().to_string();
        let primary_key_column = destination.primary_key_column().to_string();
        let ids = destination_ids.clone();
        let rows = self
            .manager
            .select_from_table(&table, move |builder| {
                builder.is_in(&primary_key_column, ids);
            })
            .fetch_all()
            .await
            .map_err(|e| from_db("many_to_many destination read failed", e))?;

        let mut by_id: BTreeMap<String, Record> = BTreeMap::new();
        for row in rows {
            let record = hydrate(destination, row.fields());
            if let Some(id) = record.primary_key(destination.primary_key_column()) {
                by_id.insert(id.to_string(), record);
            }
        }

        let mut out = Vec::with_capacity(records.len());
        for record in records {
            let key = record
                .primary_key(relationship.source_attribute())
                .or_else(|| record.get(relationship.source_attribute()))
                .map(|value| value.to_string())
                .unwrap_or_default();
            let targets = links.get(&key).cloned().unwrap_or_default();
            let found: Vec<Record> = targets
                .into_iter()
                .filter_map(|target| by_id.get(&target.to_string()).cloned())
                .collect();
            out.push(LoadedRelationship::ToMany(found));
        }
        Ok(out)
    }
}

/// Add an aggregate's summary function to a query, aliased to a fixed name so
/// the caller can read the result without knowing which aggregate it asked for.
fn add_aggregate(builder: &mut QueryBuilder, aggregate: &Aggregate) {
    let attribute = aggregate.attribute();
    match aggregate.kind() {
        AggregateKind::Count => {
            builder.count_as(attribute, "soot_aggregate");
        }
        AggregateKind::Sum => {
            builder.sum_as(attribute, "soot_aggregate");
        }
        AggregateKind::Avg => {
            builder.avg_as(attribute, "soot_aggregate");
        }
        AggregateKind::Min => {
            builder.min_as(attribute, "soot_aggregate");
        }
        AggregateKind::Max => {
            builder.max_as(attribute, "soot_aggregate");
        }
    }
}

/// The database column backing an attribute, falling back to the attribute name
/// for anything the resource did not declare.
fn column_for(resource: &ResourceDef, attribute: &str) -> String {
    resource
        .find_attribute(attribute)
        .map(|found| found.column_name().to_string())
        .unwrap_or_else(|| attribute.to_string())
}

/// Turn a driver row into a `Record`, coercing every declared attribute to its
/// declared type.
///
/// Rows come back keyed either flat or nested under the table name depending on
/// whether the query selected a single table, so both shapes are handled.
fn hydrate(resource: &Arc<ResourceDef>, values: ColumnAndValue) -> Record {
    let mut record = Record::new();

    for attribute in resource.attributes() {
        let column = attribute.column_name();
        let value = values
            .get(column)
            .or_else(|| {
                values
                    .get(resource.table_name())
                    .and_then(|nested| match nested {
                        FieldValue::Object(map) => map.get(column),
                        _ => None,
                    })
            })
            .cloned();

        if let Some(value) = value {
            record.set_field_value(attribute.name(), attribute.ty().coerce(value));
        }
    }

    // Carry through anything the resource did not declare, so a joined read
    // does not silently drop columns.
    for (key, value) in values {
        if !record.contains(&key) {
            match value {
                FieldValue::Object(_) => {}
                other => record.set_field_value(&key, other),
            }
        }
    }

    record
}

/// Translate one filter into a `QueryBuilder` predicate.
fn apply_filter(builder: &mut QueryBuilder, resource: &ResourceDef, filter: &Filter) {
    let attribute = resource.find_attribute(filter.attribute());
    let column = attribute
        .map(|a| a.column_name().to_string())
        .unwrap_or_else(|| filter.attribute().to_string());
    let ty = attribute.map(|a| a.ty().clone());
    let join = match filter.join() {
        Some(FilterJoin::Or) => Some(WhereJoin::Or),
        _ => None,
    };

    let coerce = |value: &FieldValue| match &ty {
        Some(ty) => ty.coerce(value.clone()),
        None => value.clone(),
    };

    let mut single = |operator: Operator, value: &FieldValue, join: Option<WhereJoin>| {
        builder.where_operator(&column, operator, coerce(value), join);
    };

    match filter.operator() {
        FilterOperator::IsNull => {
            builder.where_operator(&column, Operator::Null, FieldValue::Null, join);
        }
        FilterOperator::IsNotNull => {
            builder.where_operator(&column, Operator::NotNull, FieldValue::Null, join);
        }
        FilterOperator::In | FilterOperator::NotIn => {
            let values: Vec<FieldValue> = filter
                .values()
                .unwrap_or_default()
                .iter()
                .map(|value| coerce(value))
                .filter(|value| !is_nil(value))
                .collect();
            let operator = if filter.operator() == FilterOperator::In {
                Operator::In
            } else {
                Operator::NotIn
            };
            // A dirtybase value list is flattened into the bind parameters, so
            // an empty list is a query that matches nothing rather than a
            // malformed one.
            if values.is_empty() {
                match filter.operator() {
                    FilterOperator::In => {
                        builder.where_operator(&column, Operator::Null, FieldValue::Null, join);
                    }
                    _ => {
                        builder.where_operator(&column, Operator::NotNull, FieldValue::Null, join);
                    }
                }
                return;
            }
            builder.where_operator(&column, operator, FieldValue::Array(values), join);
        }
        _ => {
            let Some(value) = filter.value() else {
                return;
            };
            let operator = match filter.operator() {
                FilterOperator::Eq => Operator::Equal,
                FilterOperator::NotEq => Operator::NotEqual,
                FilterOperator::Gt => Operator::Greater,
                FilterOperator::Gte => Operator::GreaterOrEqual,
                FilterOperator::Lt => Operator::Less,
                FilterOperator::Lte => Operator::LessOrEqual,
                FilterOperator::Like => Operator::Like,
                _ => Operator::Equal,
            };
            single(operator, value, join);
        }
    }
}

/// Check the constraints of a record after a write, which is the data layer's
/// last chance to reject a value that slipped past validation.
pub fn check_row_constraints(resource: &Arc<ResourceDef>, record: &Record) -> Result<()> {
    let mut errors = ErrorList::new();
    for attribute in resource.attributes() {
        let Some(value) = record.get(attribute.name()) else {
            continue;
        };
        if let Err(found) = attribute.constraint().check(attribute.name(), &value) {
            errors.add_errors(found);
        }
    }
    errors.into_result()
}
