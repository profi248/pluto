use rumqttc::{ AsyncClient, MqttOptions, EventLoop, QoS };

use std::time::Duration;
use std::sync::Arc;

use crate::{ topics::Request, client::Client };
use crate::prelude::*;

pub const MQTT_CLIENT_ID: &'static str = "coordinator";

pub struct Coordinator {
    client: Client
}

impl Coordinator {
    pub async fn new(
        host: impl Into<String>,
        port: u16,
        username: impl Into<String>,
        password: impl Into<String>,
        handler: Arc<IncomingHandler>
    ) -> Result<(Self, EventLoop)> {
        let mut options = MqttOptions::new(MQTT_CLIENT_ID, host, port);
        options.set_keep_alive(Duration::from_secs(5));
        options.set_credentials(username, password);

        let (client, event_loop) = AsyncClient::new(options, 100);
        client.subscribe("coordinator/#", QoS::AtMostOnce).await?;

        let client = Client::new(client, handler);

        Ok((Self { client }, event_loop))
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}
