table! {
    backup_job (job_id) {
        job_id -> Integer,
        name -> Text,
        created -> BigInt,
        last_ran -> Nullable<BigInt>,
    }
}

table! {
    backup_job_path (path_id) {
        path_id -> Integer,
        job_id -> Integer,
        path_type -> Integer,
        path -> Text,
    }
}

table! {
    blob_storage (key) {
        key -> Text,
        value -> Binary,
    }
}

table! {
    node (pubkey) {
        pubkey -> Binary,
        added -> Integer,
        last_seen -> Nullable<Integer>,
        pinned -> Integer,
        label -> Nullable<Text>,
    }
}

joinable!(backup_job_path -> backup_job (job_id));

allow_tables_to_appear_in_same_query!(
    backup_job,
    backup_job_path,
    blob_storage,
    node,
);
