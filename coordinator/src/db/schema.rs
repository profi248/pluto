table! {
    backup_job (backup_job_id) {
        backup_job_id -> Int8,
        node_id -> Int8,
        created -> Timestamptz,
        last_ran -> Nullable<Timestamptz>,
        total_size -> Nullable<Int8>,
        name -> Varchar,
    }
}

table! {
    node (node_id) {
        node_id -> Int8,
        pubkey -> Bytea,
        first_seen -> Timestamptz,
        last_seen -> Timestamptz,
    }
}

table! {
    node_session (session_token) {
        session_token -> Bytea,
        node_id -> Int8,
        created -> Timestamptz,
    }
}

table! {
    node_storage_mapping (mapping_id) {
        mapping_id -> Int8,
        backup_job_id -> Int8,
        to_node -> Int8,
        data_size -> Nullable<Int8>,
        created -> Timestamptz,
        last_used -> Nullable<Timestamptz>,
    }
}

joinable!(backup_job -> node (node_id));
joinable!(node_session -> node (node_id));
joinable!(node_storage_mapping -> backup_job (backup_job_id));
joinable!(node_storage_mapping -> node (to_node));

allow_tables_to_appear_in_same_query!(
    backup_job,
    node,
    node_session,
    node_storage_mapping,
);
