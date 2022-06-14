use rumqttc::{ AsyncClient, QoS };

use std::{ sync::Arc, time::Duration };

use crate::topics::Request;
use crate::prelude::*;

/// [MQTT client](AsyncClient) which connects to an MQTT broker.
///
/// Contains a [message handler](IncomingHandler), which automatically
/// forwards messages on topics which are being listened to, back to each
/// awaiting context. This minimises the amount of handler callbacks needed,
/// as chains of messages and requests can be handled in one place.
#[derive(Clone)]
pub struct Client {
    client: AsyncClient,
    handler: Arc<IncomingHandler>,
}

impl Client {
    /// Constructs a new client with an [MQTT client](AsyncClient) and [message handler](IncomingHandler).
    pub fn new(client: AsyncClient, handler: Arc<IncomingHandler>) -> Self {
        Self { client, handler }
    }

    /// Sends one MQTT message.
    /// This method will not wait for a response.
    pub async fn send<M: MessageTrait>(&self,
        topic: String,
        message: impl Into<MessageVariant<M>>,
        qos: QoS,
        retain: bool
    ) -> Result<()> {
        let message = message.into();
        let payload = message.write_to_bytes()?;

        self.client.publish(
            topic,
            qos,
            retain,
            payload
        ).await?;

        Ok(())
    }

    /// Sends one MQTT message request and awaits its response.
    ///
    /// Since this method waits for a response, it can only be
    /// called if the input message is a [`Request`].
    ///
    /// # Arguments
    ///
    /// - `listen_topic` - The topic to listen to for response messages.
    /// - `expects_encrypted` - Whether this request expects an encrypted
    /// message as a response. Note that if this is `true`, and the response
    /// message is not encrypted, this method will return `Err`.
    pub async fn send_and_listen<R: Request>(&self,
        topic: String,
        request: impl Into<MessageVariant<R>>,
        qos: QoS,
        retain: bool,
        listen_topic: String,
        expects_encrypted: bool,
        timeout: Duration,
    ) -> Result<MessageVariant<R::Response>> {
        let response_future = self.handler.listen(listen_topic, expects_encrypted, timeout).await.unwrap();

        self.send(
            topic,
            request,
            qos,
            retain
        ).await?;

        Ok(response_future.await?)
    }
}
