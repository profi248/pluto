use chrono::DateTime;
use chrono::offset::Utc;
use diesel::{QueryDsl, RunQueryDsl};

use crate::db::{ Database, schema::node_session, Result };

#[derive(Queryable, Debug, Identifiable, Insertable)]
#[primary_key(session_token)]
#[table_name = "node_session"]
pub struct NodeSession {
    pub session_token: Vec<u8>,
    pub node_id: i64,
    pub created: DateTime<Utc>,
}

impl Database {
    pub async fn begin_node_session(&self, session_token: Vec<u8>, node_id: i64) -> Result<()> {
        self.pool.get().await?.interact(move |conn| {
            diesel::insert_into(node_session::table).values(NodeSession {
                session_token,
                node_id,
                created: Utc::now()
            }).execute(conn)
        }).await?
            .map(|_| ())
            .map_err(Into::into)
    }

    pub async fn get_node_session(&self, session_token: Vec<u8>) -> Result<NodeSession> {
        self.pool.get().await?.interact(|conn| {
            node_session::table.find(session_token)
                .get_result(conn)
        }).await?
            .map_err(Into::into)
    }
}
