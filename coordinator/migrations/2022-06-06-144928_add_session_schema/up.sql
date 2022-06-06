-- store session tokens
create table node_sessions
(
    session_token bytea
        constraint node_sessions_pk
        primary key,
    node_id bigint not null
        constraint node_sessions_node_node_id_fk
        references node,
    created timestamptz not null
);
