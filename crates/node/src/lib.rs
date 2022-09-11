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

#[derive(Debug)]
pub enum NodeError {
    RequestError(Error),
    ResponseError(ErrorType),
    DatabaseError(diesel::result::Error),
    CryptoError,
    ParseError,
    ClientError,
    NotFound,
}

impl From<pluto_network::Error> for NodeError {
    fn from(e: Error) -> Self {
        Self::RequestError(e)
    }
}

impl From<diesel::result::Error> for NodeError {
    fn from(err: diesel::result::Error) -> Self {
        Self::DatabaseError(err)
    }
}
