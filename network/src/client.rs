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

    pub async fn send<R: Request>(&self, topic: String, request: R, qos: QoS, retain: bool, timeout: Duration) -> Result<R::Response> {
        let payload = request.write_to_bytes()?;

        let initial_state = TypeId::of::<R::Response>() == TypeId::of::<()>();

        let response_future = self.handler.listen(topic.clone(), timeout, initial_state).await.unwrap();

        self.client.publish(
            topic,
            qos,
            retain,
            payload
        ).await?;

        Ok(response_future.await?)
    }
}
