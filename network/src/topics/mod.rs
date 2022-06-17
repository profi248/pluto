mod auth;

use protobuf::Message as MessageTrait;

use pluto_macros::define_topics;

use crate::protos::auth::AuthNodeInit;

define_topics! {
    Coordinator {
        Auth -> "coordinator/auth" => AuthNodeInit
    },
    Node {
        Auth -> "node/{id}/auth"
    }
}

/// Defines that a message type is a request, and expects
/// a given message type as a response.
pub trait Request: MessageTrait {
    /// The response message type expected by this request message.
    type Response: MessageTrait;

    /// Simple function to construct a default response object.
    fn response() -> Self::Response {
        Default::default()
    }
}
