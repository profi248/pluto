mod handlers;
pub mod node;
pub mod auth;
pub mod db;
pub mod utils;
pub mod config;
pub mod backup_job;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate lazy_static;


use pluto_network::Error;
use pluto_network::protos::shared::error_response::ErrorType;
use crate::NodeError::RequestError;

#[derive(Debug)]
pub enum NodeError {
    RequestError(Error),
    ResponseError(ErrorType),
    CryptoError,
    ParseError,
    ClientError
}

impl From<pluto_network::Error> for NodeError {
    fn from(e: Error) -> Self {
        RequestError(e)
    }
}
