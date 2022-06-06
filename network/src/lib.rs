#[macro_use]
extern crate tracing;

pub use rumqttc;

pub mod coordinator;
pub mod topics;
pub mod node;

pub mod protos;

mod error {
    use rumqttc::ClientError as MqttError;

    pub(crate) type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("{0}")]
        Mqtt(#[from] MqttError),
    }
}
pub use error::*;

pub mod prelude {
    pub use crate::error::*;

    pub use rumqttc;
}
