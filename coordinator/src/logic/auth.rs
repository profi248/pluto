use crate::Database;
use crate::db::models::Node;
use crate::coordinator::Coordinator;

pub const PUBKEY_LEN: usize = 32;

pub enum AuthError {
    DatabaseError,
    InvalidPubkey,
    AlreadyRegistered
}

impl Coordinator {
    pub async fn add_node_pubkey(db: &Database, pubkey: Vec<u8>) -> Result<Node, AuthError> {
        if pubkey.len() != PUBKEY_LEN { return Err(AuthError::InvalidPubkey) }

        let node = match db.get_node_from_pubkey(pubkey.clone()).await {
            Ok(Some(_)) => { return Err(AuthError::AlreadyRegistered); },
            Ok(None) => {
                match db.add_node(pubkey).await {
                    Ok(node) => node,
                    Err(_) => { return Err(AuthError::DatabaseError); }
                }
            },
            Err(_) => { return Err(AuthError::DatabaseError); }
        };

        Ok(node)
    }
}
