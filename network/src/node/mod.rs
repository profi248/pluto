use crate::{
    Result, topics::{ Topic, CoordinatorTopic },
};

pub mod key;

pub struct Node {
    connection: mqtt::AsyncClient
}

impl Node {
    pub async fn new(server: impl Into<String>) -> Result<Self> {
        let connection = mqtt::CreateOptionsBuilder::new()
            .server_uri(server)
            .client_id("")
            .mqtt_version(5)
            .create_client()?;

        trace!("step 1");

        let response = connection.connect(None).await?;

        if let Some(response) = response.connect_response() {
            info!(
                "Connected to broker at {}. MQTT version {}. {}",
                response.server_uri,
                response.mqtt_version,
                if response.session_present { "Session already present." } else { "" }
            );
        }

        Ok(Self {
            connection
        })
    }

    pub async fn register_to_network(&self, keys: &key::Keys) -> Result<()> {
        self.connection
            .publish(mqtt::Message::new(
                CoordinatorTopic::RegisterNode,
                "fsdfdf",
                0
            )).await?;

        Ok(())
    }
}
