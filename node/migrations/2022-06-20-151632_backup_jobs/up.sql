create table backup_job (
     job_id   INTEGER not null constraint backup_jobs_pk primary key autoincrement,
     name     TEXT not null,
     created  INTEGER not null,
     last_ran INTEGER
);

create unique index backup_job_name_uindex
    on backup_job (name);

create table backup_job_path (
     path_id   INTEGER not null constraint backup_job_path_pk primary key autoincrement,
     job_id    INTEGER not null constraint backup_job_path_backup_job_job_id_fk
         references backup_job(job_id)
         on update cascade on delete cascade,
     -- 0 = folder path, 1 = exclusion pattern
     path_type INTEGER not null check ( path_type in (0, 1) ),
     -- max length is Windows filesystem limit in Unicode characters
     path      TEXT not null check ( length(path) > 0 and length(path) < 32760 )
);

create unique index backup_path_uindex
    on backup_job_path (job_id, path, path_type);
