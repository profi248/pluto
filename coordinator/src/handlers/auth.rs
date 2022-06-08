use pluto_network::{ prelude::*, topics::*, client::Client };
use pluto_network::protos::auth::{
    *,
    auth_coordinator_challenge::*
};
use crate::DATABASE;
use crate::rumqttc::QoS;

pub struct AuthHandler;

#[async_trait::async_trait]
impl Handler for AuthHandler {
    fn topic(&self) -> Topic {
        topic!(Coordinator::Auth).into()
    }

    async fn handle(&self, message: Message, client: Client) {
        let init_msg: AuthNodeInit = match message.parse() {
            Some(m) => m,
            None => { debug!("invalid message"); return; },
        };

        debug!("{:?}", init_msg.pubkey);
        let node = DATABASE.get().unwrap().get_node_from_pubkey(init_msg.pubkey).await;
        //debug!("{:?}", node.unwrap());

        let mut challenge_msg_wrapper = AuthNodeInit::response();
        let mut challenge_msg = Challenge::default();

        let mut challenge_bytes: [u8; 32] = [0; 32];
        getrandom::getrandom(&mut challenge_bytes).unwrap();
        challenge_msg.challenge = challenge_bytes.to_vec();

        challenge_msg_wrapper.challenge_status = Some(Challenge_status::Challenge(challenge_msg));

        let msg = match client.send(
            topic!(Node::Auth).topic("a".to_owned()),
            challenge_msg_wrapper,
            QoS::AtMostOnce,
            false,
            std::time::Duration::from_secs(10)
        ).await {
            Err(e) => { debug!("{e:?}"); return; },
            Ok(msg) => msg
        };
    }
}
