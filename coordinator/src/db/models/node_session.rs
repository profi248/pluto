use chrono::DateTime;
use chrono::offset::Utc;

use crate::db::{ schema::node_session };

#[derive(Queryable, Debug, Identifiable)]
#[primary_key(session_token)]
#[table_name(node_session)]
pub struct NodeSession {
    pub session_token: Vec<u8>,
    pub node_id: i64,
    pub created: DateTime<Utc>,
}
