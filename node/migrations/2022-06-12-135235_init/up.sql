create table blob_storage
(
    key TEXT not null
        constraint "primary"
        primary key,
    value BLOB not null
);
