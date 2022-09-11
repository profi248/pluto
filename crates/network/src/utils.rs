pub fn get_node_topic_id(pubkey_bytes: Vec<u8>) -> String {
    base64::encode_config(
        pubkey_bytes,
        base64::URL_SAFE_NO_PAD
    )
}
