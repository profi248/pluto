#[macro_use]
extern crate tracing;

pub use rumqttc;

pub mod coordinator;
pub mod handler;
pub mod topics;
pub mod node;
pub mod client;

pub mod protos;

mod error {
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
    }
}
pub use error::{Error, Result};

pub mod prelude {
    pub use crate::error::*;

    pub use crate::handler::*;

    pub use rumqttc;
}
