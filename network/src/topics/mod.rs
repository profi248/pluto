mod auth;

use rumqttc::AsyncClient;
use pluto_macros::define_topics;

use crate::protos::auth::AuthNodeInit;

define_topics! {
    Coordinator {
        Auth -> "coordinator/auth" => AuthNodeInit
    },
    Node {
        Auth -> "node/{id}/auth"
        // "node/3478347384/auth" -> Node::Auth
        // {} -> [[:alnum:]]
    }
}

pub trait Request: protobuf::Message {
    type Response: protobuf::Message;

    fn response() -> Self::Response {
        Default::default()
    }
}
