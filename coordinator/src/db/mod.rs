pub mod schema;
pub mod models;

use deadpool_diesel::{
    Pool, Runtime, PoolError,
    postgres::{ Manager, InteractError }
};

use diesel::result::Error as DieselError;

pub struct Database {
    pool: Pool<Manager>
}

impl Database {
    pub fn new(url: impl Into<String>) -> Self {
        let manager = Manager::new(url, Runtime::Tokio1);
        let pool = Pool::builder(manager)
            .max_size(100)
            // todo: no clean way as of now to return this.
            .build().expect("Failed to connect to the database");

        Self { pool }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    PoolError(#[from] PoolError),
    #[error("{0}")]
    QueryError(#[from] DieselError),
    #[error("{0}")]
    Panic(#[from] InteractError),
}