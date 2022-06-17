#[allow(unused_imports)]
#[macro_use]
extern crate tracing;

pub use rumqttc;
pub use x25519_dalek;

pub mod handler;
pub mod topics;
pub mod client;
pub mod key;
pub mod message;
pub mod utils;

pub mod protos;

pub mod error {
    pub use rumqttc::ClientError as MqttError;
    pub use protobuf::Error as ProtobufError;

    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("{0}")]
        Mqtt(#[from] MqttError),
        #[error("{0}")]
        Protobuf(#[from] ProtobufError),
        #[error("Timed out.")]
        TimedOut,
        #[error("{0}")]
        HandlerError(#[from] crate::handler::HandlerError),
    }
}

pub use error::{Error, Result};

pub mod prelude {
    pub use crate::error::*;
    pub use crate::handler::*;
    pub use crate::message::*;

    pub use rumqttc;
    pub use x25519_dalek;

    pub use protobuf::Message as MessageTrait;

    pub(crate) use std::result::Result as StdResult;

    // import macro
    pub use crate::topics::topic;
}
