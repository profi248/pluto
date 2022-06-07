use chrono::DateTime;
use chrono::offset::Utc;
use diesel::prelude::*;

use crate::db::schema::node_storage_mapping;

#[derive(Queryable, Debug, Identifiable)]
#[primary_key(mapping_id)]
#[table_name = "node_storage_mapping"]
pub struct NodeStorageMapping {
    pub mapping_id: i64,
    pub backup_job_id: i64,
    pub to_node: i64,
    pub data_size: Option<i64>,
    pub created: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}
