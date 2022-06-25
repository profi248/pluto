use std::path::PathBuf;

use pluto_network::client::Client;
use pluto_network::key::Keys;
use pluto_network::rumqttc::QoS;

pub const PLUTO_DIR: &str = "pluto";
pub const LOG_FILE: &str = "log.txt";
pub const DB_FILE: &str = "pluto.db";

pub fn get_pluto_dir() -> PathBuf {
    let mut path = dirs::config_dir().unwrap();
    path.push(PLUTO_DIR);

    path
}

pub fn get_db_file_path() -> String {
    let mut path = get_pluto_dir();
    path.push(DB_FILE);

    path.to_str().unwrap().to_owned()
}

pub fn get_log_file_path() -> String {
    let mut path = get_pluto_dir();
    path.push(LOG_FILE);

    path.to_str().unwrap().to_owned()
}

pub fn setup_dirs() {
    let path = get_pluto_dir();

    if !path.exists() { std::fs::create_dir(&path).unwrap(); }
    if !path.is_dir() { panic!("{} is a file.", PLUTO_DIR); }
}

pub async fn subscribe_to_topics(client: Client, keys: &Keys) -> Option<()> {
    let node_topic_id = pluto_network::utils::get_node_topic_id(keys.public_key().as_bytes().to_vec());
    client.client().subscribe(format!("node/{node_topic_id}/#"), QoS::AtMostOnce).await.ok()?;

    Some(())
}
