use chrono::DateTime;
use chrono::offset::Utc;
use diesel::prelude::*;

use crate::db::{ Database, schema::node, Result };

#[derive(Queryable, Debug, Identifiable)]
#[primary_key(node_id)]
#[table_name(node)]
pub struct Node {
    pub node_id: i64,
    pub pubkey: Vec<u8>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>
}

#[derive(Insertable)]
#[table_name(node)]
pub struct NodeInsert {
    pub pubkey: Vec<u8>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>
}

impl Database {
    pub async fn get_node_from_pubkey(&self, pubkey: Vec<u8>) -> Result<Option<Node>> {
        self.pool.get().await?.interact(|conn|
            node::table.filter(node::pubkey.eq(pubkey))
            .first(conn).optional()
        ).await?
         .map_err(Into::into)
    }

    pub async fn add_node(&self, pubkey: Vec<u8>) -> Result<Node> {
        self.pool.get().await?.interact(|conn| {
            let time = Utc::now();

            diesel::insert_into(node::table).values(NodeInsert {
                pubkey,
                first_seen: time.clone(),
                last_seen: time
            }).returning(node::all_columns)
              .get_result(conn)
        }).await?
          .map_err(Into::into)
    }

    pub async fn node_update_last_seen(&self, node_id: i64) -> Result<()> {
        self.pool.get().await?.interact(move |conn|
            diesel::update(node::table).filter(node::node_id.eq(node_id))
                .set(node::last_seen.eq(Utc::now()))
                .execute(conn)
        ).await?
         .map(|_| ())
         .map_err(Into::into)
    }
}
