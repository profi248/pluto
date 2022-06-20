use diesel::prelude::*;
use crate::db::{ Database };
use crate::db::schema:: backup_job_path;

#[derive(Queryable, Debug, Identifiable)]
#[primary_key(path_id)]
#[table_name = "backup_job_path"]
pub struct BackupJobPath {
    pub path_id: i32,
    pub job_id: i32,
    pub path_type: i32,
    pub path: String,
}
