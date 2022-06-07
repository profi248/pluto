use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS };

use std::time::Duration;
use std::sync::Arc;

pub const MQTT_NODE_USERNAME: &'static str = "node";
pub const MQTT_NODE_PASSWORD: &'static str = "node";

use crate::prelude::*;

use crate::{
    Result, topics::Request, client::Client
};

pub mod key;

pub struct Node {
    client: Client
}

impl Node {
    pub async fn new(host: impl Into<String>, port: u16, handler: Arc<IncomingHandler>) -> Result<(Self, EventLoop)> {
        let mut options = MqttOptions::new("a", host, port);
        options.set_keep_alive(Duration::from_secs(60));
        options.set_credentials(MQTT_NODE_USERNAME, MQTT_NODE_PASSWORD);

        let (client, event_loop) = AsyncClient::new(options, 100);

        let client = Client::new(client, handler);

        Ok((Self { client }, event_loop))
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}
