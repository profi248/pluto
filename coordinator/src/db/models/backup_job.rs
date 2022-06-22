use diesel::prelude::*;
use chrono::prelude::*;

use crate::db::{ Database, schema::backup_job, Result };

#[derive(Queryable, Debug, Identifiable)]
#[primary_key(backup_job_id)]
#[table_name = "backup_job"]
pub struct BackupJob {
    pub backup_job_id: i64,
    pub node_id: i64,
    pub local_job_id: i32,
    pub created: DateTime<Utc>,
    pub last_ran: Option<DateTime<Utc>>,
    pub total_size: Option<i64>,
    pub name: String,
}

impl Default for BackupJob {
    fn default() -> Self {
        Self {
            backup_job_id: 0,
            node_id: 0,
            local_job_id: 0,
            created: Utc::now(),
            last_ran: None,
            total_size: None,
            name: "".to_string()
        }
    }
}

#[derive(Insertable)]
#[table_name = "backup_job"]
pub struct BackupJobInsert {
    pub node_id: i64,
    pub local_job_id: i32,
    pub created: DateTime<Utc>,
    pub name: String,
}

impl Database {
    pub async fn create_backup_job(&self, node_id: i64, local_job_id: i32, created: DateTime<Utc>, name: String) -> Result<BackupJob> {
        self.pool.get().await?.interact(move |conn| {
            diesel::insert_into(backup_job::table).values(BackupJobInsert {
                node_id,
                local_job_id,
                created,
                name
            }).returning(backup_job::all_columns)
                .get_result(conn)
        }).await?
            .map_err(Into::into)
    }

    pub async fn update_backup_job(&self, node_id: i64, local_job_id: i32, total_size: Option<i64>, last_ran: Option<DateTime<Utc>>, name: String) -> Result<()> {
        self.pool.get().await?.interact(move |conn| {
            diesel::update(backup_job::table).filter(
                backup_job::node_id.eq(node_id).and(backup_job::local_job_id.eq(local_job_id))
            ).set((backup_job::last_ran.eq(last_ran),
                backup_job::total_size.eq(total_size),
                backup_job::name.eq(name)))
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

    pub async fn get_backup_job_by_local_id(&self, node_id: i64, local_job_id: i32) -> Result<Option<BackupJob>> {
        self.pool.get().await?.interact(move |conn| {
            backup_job::table.filter(
                backup_job::node_id.eq(node_id).and(backup_job::local_job_id.eq(local_job_id))
            ).first(conn).optional()
        }).await?
            .map_err(Into::into)
    }
}
