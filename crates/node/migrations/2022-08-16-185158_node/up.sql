create table node (
  pubkey BLOB not null,
  added INTEGER not null,
  last_seen INTEGER,
  pinned INTEGER not null check ( pinned in (0, 1) ),
  label TEXT,
  primary key (pubkey)
);
