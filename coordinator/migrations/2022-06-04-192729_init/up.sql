-- nodes
create table node (
    node_id bigserial
        constraint node_pk
        primary key,
    pubkey     bytea not null,
    first_seen timestamptz not null,
    last_seen  timestamptz not null
);

create unique index node_pubkey_uindex
    on node (pubkey);

-- backup jobs
create table backup_job (
    backup_job_id bigserial
        constraint backup_job_pk
        primary key,
    node_id bigint not null
        constraint backup_job_node_node_id_fk
        references node
        on update restrict on delete restrict,
    created       timestamptz not null,
    last_ran      timestamptz,
    total_size    bigint,
    name          varchar(255) not null
);

create index backup_job_node_id_index
    on backup_job (node_id);

create unique index backup_job_node_id_name_uindex
    on backup_job (node_id, name);

-- nodes storage mappings
create table node_storage_mapping (
    mapping_id bigserial
        constraint node_storage_mapping_pk
        primary key,
    backup_job_id bigint not null
        constraint node_storage_mapping_backup_job_backup_job_id_fk
        references backup_job
        on update cascade on delete cascade,
    to_node bigint not null
        constraint node_storage_mapping_node_node_id_fk
        references node
        on update restrict on delete restrict,
    data_size     bigint,
    created       timestamptz not null,
    last_used     timestamptz
);

create unique index node_storage_mapping_job_id_to_node_uindex
    on node_storage_mapping (backup_job_id, to_node);
