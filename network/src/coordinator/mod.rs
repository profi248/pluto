use crate::Result;

pub struct Coordinator {
    connection: mqtt::AsyncClient
}

impl Coordinator {
    pub async fn new(username: impl Into<String>, password: impl Into<String>) -> Result<Self> {
        let connection = mqtt::CreateOptionsBuilder::new()
            .server_uri("tcp://localhost:1883")
            .client_id("coordinator")
            .mqtt_version(5)
            .create_client()?;

        let response = connection.connect(mqtt::ConnectOptionsBuilder::new()
            .user_name(username)
            .password(password)
            .clean_start(true)
            .finalize()
        ).await?;

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


}
