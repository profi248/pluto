create table backup_job (
     job_id   INTEGER not null constraint backup_jobs_pk primary key autoincrement,
     name     TEXT not null,
     created  INTEGER not null,
     last_run INTEGER
);

create unique index backup_job_name_uindex
    on backup_job (name);

create table backup_job_path (
     path_id   INTEGER not null constraint backup_job_path_pk primary key autoincrement,
     job_id    INTEGER not null constraint backup_job_path_backup_job_job_id_fk
         references backup_job(job_id)
         on update cascade on delete cascade,
     path_type INTEGER not null,
     path      TEXT not null
);
