use warp::{ reply, http::StatusCode, reply::Json };
use serde::{ Serialize, Deserialize };
use sha2::{ Sha256, Digest };

use pluto_node::db::Database;
use pluto_macros::reject;

use super::generate_error;

#[derive(Serialize)]
pub struct NodeJson {
    pub pubkey: String,
    pub pubkey_hash: String,
    pub added: i32,
    pub last_seen: Option<i32>,
    pub pinned: i32,
    pub label: Option<String>,
}

#[reject]
pub async fn get_nodes() -> Result<impl warp::Reply, reply::WithStatus<Json>> {
    let db = Database::new();
    let nodes_raw = db.get_nodes()
        .map_err(|e| generate_error(format!("Error: {e:?}"), StatusCode::INTERNAL_SERVER_ERROR))?;

    let nodes: Vec<NodeJson> = nodes_raw.iter().map(|node| {
        NodeJson {
            pubkey: hex::encode(node.pubkey.clone()),
            pubkey_hash: hex::encode(Sha256::digest(&node.pubkey)),
            added: node.added,
            last_seen: node.last_seen,
            pinned: node.pinned,
            label: node.label.clone(),
        }
    }).collect();

    Ok(reply::json(&nodes))
}
