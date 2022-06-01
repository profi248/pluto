#[macro_use]
extern crate tracing;
extern crate core;

pub mod coordinator;
pub mod topics;
pub mod node;

mod error {
    pub use mqtt::Error as MqttError;

    pub(crate) type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("{0}")]
        Mqtt(#[from] MqttError)
    }
}
pub use error::*;

pub mod prelude {
    pub use crate::error::*;
}