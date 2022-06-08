use rumqttc::{ AsyncClient, QoS };

use std::{ sync::Arc, time::Duration, any::TypeId };

use crate::topics::Request;
use crate::prelude::*;

#[derive(Clone)]
pub struct Client {
    client: AsyncClient,
    handler: Arc<IncomingHandler>,
}

impl Client {
    pub fn new(client: AsyncClient, handler: Arc<IncomingHandler>) -> Self {
        Self { client, handler }
    }

    pub async fn send<M: MessageTrait>(&self, topic: String, message: M, qos: QoS, retain: bool) -> Result<()> {
        let payload = message.write_to_bytes()?;

        self.client.publish(
            topic,
            qos,
            retain,
            payload
        ).await?;

        Ok(())
    }

    pub async fn send_and_listen<R: Request>(&self, topic: String, request: R, qos: QoS, retain: bool, timeout: Duration) -> Result<R::Response> {
        let response_future = self.handler.listen(topic.clone(), timeout).await.unwrap();

        self.send(
            topic,
            request,
            qos,
            retain
        ).await?;

        Ok(response_future.await?)
    }
}
