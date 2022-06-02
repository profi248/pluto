use rumqttc::{ AsyncClient, MqttOptions, EventLoop, QoS };

use std::time::Duration;

use crate::Result;

pub struct Coordinator {
    client: AsyncClient,
}

impl Coordinator {
    pub async fn new(username: impl Into<String>, password: impl Into<String>) -> Result<(Self, EventLoop)> {
        let mut options = MqttOptions::new("coordinator", "localhost", 1883);
        options.set_keep_alive(Duration::from_secs(5));
        // options.set_credentials(username, password);

        let (client, event_loop) = AsyncClient::new(options, 100);
        client.subscribe("#", QoS::AtMostOnce).await.unwrap();

        Ok((Self { client }, event_loop))
    }
}
