use rumqttc::{ AsyncClient, EventLoop, MqttOptions, QoS };

use std::time::Duration;

pub const MQTT_NODE_USERNAME: &'static str = "node";
pub const MQTT_NODE_PASSWORD: &'static str = "node";

use crate::{
    Result, topics::{ Topic, CoordinatorTopic },
};

pub mod key;

pub struct Node {
    client: AsyncClient
}

impl Node {
    pub async fn new(host: impl Into<String>, port: u16) -> Result<(Self, EventLoop)> {
        let mut options = MqttOptions::new("a", host, port);
        options.set_keep_alive(Duration::from_secs(60));
        options.set_credentials(MQTT_NODE_USERNAME, MQTT_NODE_PASSWORD);

        let (client, event_loop) = AsyncClient::new(options, 100);

        Ok((Self { client }, event_loop))
    }

    pub async fn register_to_network(&self, keys: &key::Keys) -> Result<()> {
        self.client
            .publish(
                CoordinatorTopic::RegisterNode,
                QoS::ExactlyOnce,
                false,
                "lol"
            ).await
             .map_err(Into::into)
    }
}
