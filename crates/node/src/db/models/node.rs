use diesel::prelude::*;

use crate::db::Database;
use crate::db::schema::node;

#[derive(Queryable, Debug, Identifiable)]
#[primary_key(pubkey)]
#[table_name = "node"]
pub struct Node {
    pub pubkey: Vec<u8>,
    pub added: i32,
    pub last_seen: Option<i32>,
    pub pinned: i32,
    pub label: Option<String>,
}

impl Database {
    pub fn get_nodes(&self) -> QueryResult<Vec<Node>> {
        node::table.load(&self.conn)
    }

    pub fn create_node(&self, pubkey: Vec<u8>, added: i32, last_seen: Option<i32>, pinned: i32, label: Option<String>) -> QueryResult<()> {
        diesel::insert_into(node::table)
            .values((
                node::pubkey.eq(pubkey),
                node::added.eq(added),
                node::last_seen.eq(last_seen),
                node::pinned.eq(pinned),
                node::label.eq(label),
            ))
            .execute(&self.conn)
            .map(|_| ())
    }

    pub fn edit_node(&self, pubkey: Vec<u8>, added: i32, last_seen: Option<i32>, pinned: i32, label: Option<String>) -> QueryResult<()> {
        diesel::update(node::table)
            .filter(node::pubkey.eq(pubkey))
            .set((
                node::added.eq(added),
                node::last_seen.eq(last_seen),
                node::pinned.eq(pinned),
                node::label.eq(label),
            ))
            .execute(&self.conn)
            .map(|_| ())
    }

    pub fn get_node(&self, pubkey: impl Into<Vec<u8>>) -> QueryResult<Option<Node>> {
        node::table.find(pubkey.into())
            .first(&self.conn)
            .optional()
    }

    pub fn delete_node(&self, pubkey: impl Into<Vec<u8>>) -> QueryResult<()> {
        diesel::delete(node::table.find(pubkey.into()))
            .execute(&self.conn)
            .map(|_| ())
    }
}
