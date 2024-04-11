use dirtybase_db_macro::DirtyTable;

#[derive(Debug, Clone, Default, DirtyTable)]
#[dirty(id = "tag", table = "core_cache_tags")]
pub struct CacheDbTagStoreEntity {
    pub(crate) tag: String,
}

#[derive(Debug, Clone, Default, DirtyTable)]
#[dirty(table = "core_cache_tag_entries", id = "core_cache_tags_id")]
pub struct CacheDbPivotEntity {
    pub(crate) core_cache_tags_id: i64,
    pub(crate) core_cache_key: String,
}
