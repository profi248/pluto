use crate::Database;
use crate::db::models::Node;

use ring::signature::{ self, KeyPair };

pub const PUBKEY_LEN: usize = 16;
pub const CHALLENGE_LEN: usize = 16;
pub const SESSION_TOKEN_LEN: usize = 16;

pub async fn add_or_find_node_pubkey(db: &Database, pubkey: Vec<u8>) -> Option<Node> {
    if pubkey.len() != PUBKEY_LEN { return None }

    // todo: handle critical database errors?
    let node = match db.get_node_from_pubkey(pubkey.clone()).await.unwrap() {
        Some(node) => node,
        None => {
            db.add_node(pubkey).await.unwrap()
        }
    };

    Some(node)
}

pub fn generate_challenge_bytes() -> Vec<u8> {
    let mut challenge_bytes: [u8; CHALLENGE_LEN] = [0; CHALLENGE_LEN];
    getrandom::getrandom(&mut challenge_bytes).unwrap();

    challenge_bytes.to_vec()
}

pub fn verify_challenge(challenge: Vec<u8>, signature: Vec<u8>, pubkey: Vec<u8>) -> Option<()> {
    let node_pubkey =
        signature::UnparsedPublicKey::new(&signature::ED25519, pubkey);

    node_pubkey.verify(&*challenge, &*signature).ok()
}

pub async fn create_session(db: &Database, node_id: i64) -> Vec<u8> {
    let mut session_token:[u8; SESSION_TOKEN_LEN] = [0; SESSION_TOKEN_LEN];
    getrandom::getrandom(&mut session_token).unwrap();
    db.begin_node_session(session_token.to_vec(), node_id).await.unwrap();

    session_token.to_vec()
}
