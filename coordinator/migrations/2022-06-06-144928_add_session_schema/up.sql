-- store session tokens
create table node_session
(
    session_token bytea
        constraint node_session_pk
        primary key,
    node_id bigint not null
        constraint node_session_node_node_id_fk
        references node,
    created timestamptz not null
);
