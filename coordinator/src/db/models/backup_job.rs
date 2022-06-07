use chrono::DateTime;
use chrono::offset::Utc;
use diesel::prelude::*;

use crate::db::{ Database, schema::backup_job, Result };

#[derive(Queryable, Debug, Identifiable)]
#[primary_key(backup_job_id)]
#[table_name = "backup_job"]
pub struct BackupJob {
    pub backup_job_id: i64,
    pub node_id: i64,
    pub created: DateTime<Utc>,
    pub last_ran: Option<DateTime<Utc>>,
    pub total_size: Option<i64>,
    pub name: String,
}

#[derive(Insertable)]
#[table_name(backup_job)]
pub struct BackupJobInsert {
    pub node_id: i64,
    pub created: DateTime<Utc>,
    pub name: String,
}

impl Database {
    pub async fn create_backup_job(&self, node_id: i64, name: String) -> Result<BackupJob> {
        self.pool.get().await?.interact(move |conn| {
            diesel::insert_into(backup_job::table).values(BackupJobInsert {
                node_id,
                created: Utc::now(),
                name
            }).returning(backup_job::all_columns)
                .get_result(conn)
        }).await?
            .map_err(Into::into)
    }

    pub async fn update_backup_job(&self, backup_job_id: i64, total_size: i64) -> Result<()> {
        self.pool.get().await?.interact(move |conn| {
            diesel::update(backup_job::table).filter(backup_job::backup_job_id.eq(backup_job_id))
                .set((backup_job::last_ran.eq(Utc::now()),
                            backup_job::total_size.eq(total_size)))
                .execute(conn)
        }).await?
            .map(|_| ())
            .map_err(Into::into)
    }

    pub async fn rename_backup_job(&self, backup_job_id: i64, name: String) -> Result<()> {
        self.pool.get().await?.interact(move |conn| {
            diesel::update(backup_job::table).filter(backup_job::backup_job_id.eq(backup_job_id))
                .set(backup_job::name.eq(name))
                .execute(conn)
        }).await?
            .map(|_| ())
            .map_err(Into::into)
    }

    pub async fn delete_backup_job(&self, backup_job_id: i64) -> Result<()> {
        self.pool.get().await?.interact(move |conn| {
            diesel::delete(backup_job::table.find(backup_job_id))
                .execute(conn)
        }).await?
            .map(|_| ())
            .map_err(Into::into)
    }

    pub async fn get_backup_jobs_by_node(&self, node_id: i64) -> Result<Vec<BackupJob>> {
        self.pool.get().await?.interact(move |conn| {
            backup_job::table.filter(backup_job::node_id.eq(node_id))
                .get_results(conn)
        }).await?
            .map_err(Into::into)
    }
}
