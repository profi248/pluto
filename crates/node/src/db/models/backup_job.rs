use diesel::prelude::*;
use chrono::{ DateTime, Utc };
use serde::{ Serialize, Deserialize };

use crate::db::{ Database, last_insert_rowid };
use crate::db::models::backup_job_path::BackupJobPath;
use crate::db::schema::{ backup_job_path, backup_job };

#[derive(Queryable, Debug, Identifiable, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[primary_key(job_id)]
#[table_name = "backup_job"]
pub struct BackupJob {
    pub job_id: i32,
    pub name: String,
    pub created: i64,
    pub last_ran: Option<i64>,
}

#[derive(Insertable, Clone, Deserialize)]
#[table_name = "backup_job"]
pub struct BackupJobInsert {
    pub name: String,
    pub created: i64,
    pub last_ran: Option<i64>,
}

impl Database {
    pub fn create_backup_job(&self, name: String, created: DateTime<Utc>) -> QueryResult<i32> {
        diesel::insert_into(backup_job::table)
            .values(BackupJobInsert {
                name,
                created: created.timestamp(),
                last_ran: None,
            })
            .execute(&self.conn)?;

        diesel::select(last_insert_rowid)
            .get_result::<i32>(&self.conn)
    }

    pub fn get_backup_job(&self, job_id: i32) -> QueryResult<Option<BackupJob>> {
        backup_job::table.find(job_id).first(&self.conn).optional()
    }

    pub fn get_backup_jobs(&self) -> QueryResult<Vec<(BackupJob, BackupJobPath)>> {
        backup_job::table
            .inner_join(backup_job_path::table)
            .order(backup_job::created.desc())
            .load(&self.conn)
    }

    pub fn update_backup_job(&self, job_id: i32, name: String, last_ran: Option<i64>) -> QueryResult<()> {
        diesel::update(backup_job::table)
            .filter(backup_job::job_id.eq(job_id))
            .set((
                backup_job::name.eq(name),
                backup_job::last_ran.eq(last_ran),
            ))
            .execute(&self.conn)
            .map(|_| ())
    }

    pub fn delete_backup_job(&self, job_id: i32) -> QueryResult<()> {
        diesel::delete(backup_job::table.find(job_id)).execute(&self.conn).map(|_| ())
    }
}
