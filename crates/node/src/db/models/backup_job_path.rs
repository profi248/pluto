use diesel::prelude::*;
use serde::{ Serialize, Deserialize };

use crate::db::{ Database, last_insert_rowid };
use crate::db::schema:: backup_job_path;

#[derive(Queryable, Debug, Identifiable, Clone)]
#[primary_key(path_id)]
#[table_name = "backup_job_path"]
pub struct BackupJobPath {
    pub path_id: i32,
    pub job_id: i32,
    pub path_type: i32,
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum PathType {
    Folder = 0,
    IgnorePattern = 1
}

impl TryFrom<i32> for PathType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PathType::Folder),
            1 => Ok(PathType::IgnorePattern),
            _ => Err(())
        }
    }
}

impl Database {
    pub fn create_backup_job_path(&self, job_id: i32, path: String, path_type: PathType) -> QueryResult<i32> {
        diesel::insert_into(backup_job_path::table)
            .values((
                backup_job_path::job_id.eq(job_id),
                backup_job_path::path_type.eq(path_type as i32),
                backup_job_path::path.eq(path),
            ))
            .execute(&self.conn)?;

        diesel::select(last_insert_rowid)
            .get_result::<i32>(&self.conn)
    }

    pub fn update_backup_job_path(&self, path_id: i32, path: String, path_type: PathType) -> QueryResult<()> {
        diesel::update(backup_job_path::table)
            .filter(backup_job_path::path_id.eq(path_id))
            .set((
                backup_job_path::path.eq(path),
                 backup_job_path::path_type.eq(path_type as i32)
             ))
            .execute(&self.conn)
            .map(|_| ())
    }

    pub fn delete_backup_job_path(&self, path_id: i32) -> QueryResult<()> {
        diesel::delete(backup_job_path::table.find(path_id)).execute(&self.conn).map(|_| ())
    }

    pub fn backup_job_has_path_id(&self, job_id: i32, path_id: i32) -> QueryResult<bool> {
        backup_job_path::table
            .filter(backup_job_path::job_id.eq(job_id))
            .filter(backup_job_path::path_id.eq(path_id))
            .first(&self.conn)
            .optional()
            .map(|x: Option<BackupJobPath>| x.is_some())
    }

    pub fn get_paths_for_backup_job(&self, job_id: i32) -> QueryResult<Vec<BackupJobPath>> {
        backup_job_path::table
            .filter(backup_job_path::job_id.eq(job_id))
            .load(&self.conn)
    }
}
