use rumqttc::{ AsyncClient, EventLoop, MqttOptions, QoS };
use x25519_dalek::PublicKey;

use std::time::Duration;
use std::sync::Arc;

pub const MQTT_NODE_USERNAME: &'static str = "node";
pub const MQTT_NODE_PASSWORD: &'static str = "node";

lazy_static! {
    pub static ref COORDINATOR_PUBKEY: PublicKey = {
        let bytes = base64::decode(crate::config::COORDINATOR_PUBKEY).unwrap();
        let bytes: [u8; 32] = bytes.try_into().unwrap();
        PublicKey::from(bytes)
    };
}

use pluto_network::prelude::*;

use pluto_network::{
    Result, client::Client
};

pub struct Node {
    client: Client
}

impl Node {
    pub async fn new(host: impl Into<String>, port: u16, client_id: String, handler: Arc<IncomingHandler>) -> Result<(Self, EventLoop)> {
        let mut options = MqttOptions::new(client_id.clone(), host, port);
        options.set_keep_alive(Duration::from_secs(30));
        options.set_credentials(MQTT_NODE_USERNAME, MQTT_NODE_PASSWORD);
        options.set_clean_session(true);

        let (client, event_loop) = AsyncClient::new(options, 100);

        let client = Client::new(client, handler);

        Ok((Self { client }, event_loop))
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}
