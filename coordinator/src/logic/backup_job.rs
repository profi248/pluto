use crate::{ Coordinator, Database };
use crate::db::models::BackupJob;

pub enum BackupJobError {
    DatabaseError,
    NameTooLong,
    InvalidTimestamp
}

impl Coordinator {
    pub async fn insert_or_update_backup_job(db: &Database, job: BackupJob) -> Result<(), BackupJobError> {
        if job.name.len() > 255 { return Err(BackupJobError::NameTooLong) }
        if job.created.timestamp() < 0 { return Err(BackupJobError::InvalidTimestamp) }
        if job.last_ran.is_some() && job.last_ran.unwrap().timestamp() < 0 { return Err(BackupJobError::InvalidTimestamp) }

        match db.get_backup_job_by_local_id(job.node_id, job.local_job_id).await {
            Ok(Some(_)) => {
                db.update_backup_job(job.node_id, job.local_job_id, job.total_size, job.last_ran, job.name).await.map_err(|_| BackupJobError::DatabaseError)?;
                Ok(())
            },
            Ok(None) => {
                db.create_backup_job(job.node_id, job.local_job_id, job.created, job.name).await.map_err(|_| BackupJobError::DatabaseError)?;
                Ok(())
            },
            Err(_) => {
                Err(BackupJobError::DatabaseError)
            }
        }
    }
}
