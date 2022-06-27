use diesel::prelude::*;
use chrono::{ DateTime, Utc };
use serde::{ Serialize, Deserialize };

use crate::db::{ Database, last_insert_rowid };
use crate::db::schema::backup_job;

#[derive(Queryable, Debug, Identifiable, Default, Clone, Serialize, Deserialize)]
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
    pub fn create_backup_job(name: String, created: DateTime<Utc>) -> QueryResult<i32> {
        let conn = &Self::connect();
        diesel::insert_into(backup_job::table)
            .values(BackupJobInsert {
                name,
                created: created.timestamp(),
                last_ran: None,
            })
            .execute(conn)?;

        diesel::select(last_insert_rowid)
            .get_result::<i32>(conn)
    }

    pub fn get_backup_job(job_id: i32) -> QueryResult<Option<BackupJob>> {
        backup_job::table.find(job_id).first(&Self::connect()).optional()
    }

    pub fn get_backup_jobs() -> QueryResult<Vec<BackupJob>> {
        backup_job::table.order(backup_job::created.desc()).load(&Self::connect())
    }

    pub fn update_backup_job(job_id: i32, name: String, last_ran: Option<i64>) -> QueryResult<()> {
        diesel::update(backup_job::table)
            .filter(backup_job::job_id.eq(job_id))
            .set((
                backup_job::name.eq(name),
                backup_job::last_ran.eq(last_ran),
            ))
            .execute(&Self::connect())
            .map(|_| ())
    }

    pub fn delete_backup_job(job_id: i32) -> QueryResult<()> {
        diesel::delete(backup_job::table.find(job_id)).execute(&Self::connect()).map(|_| ())
    }
}
