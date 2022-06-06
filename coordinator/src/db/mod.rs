pub mod schema;
pub mod models;

embed_migrations!();

use deadpool_diesel::{
    Pool, Runtime, PoolError,
    postgres::{ Manager, InteractError }
};

use diesel::migration::RunMigrationsError;
use diesel::result::Error as DieselError;
use diesel::prelude::*;

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

    pub async fn check_connection(&self) -> Result<()> {
        self.pool.get().await?.interact(move |conn|
            diesel::sql_query("select 1;").execute(conn)
        ).await?
            .map(|_| ())
            .map_err(Into::into)
    }

    pub async fn run_migrations(&self) -> Result<()> {
        #[derive(Default)]
        struct Logger {
            buffer: String,
        }

        impl std::io::Write for Logger {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                let string = String::from_utf8_lossy(buf);

                self.buffer.push_str(&string);

                let mut lines: Vec<&str> = self.buffer.split_inclusive('\n')
                    // Ignore empty lines.
                    .filter(|&s| s != "\n")
                    .collect();

                let last_line = match lines.pop() {
                    Some(l) => l,
                    None => return Ok(buf.len()),
                };

                for line in lines {
                    info!("{}", line.trim_end());
                }

                if last_line.ends_with('\n') {
                    info!("{}", last_line.trim_end());
                    self.buffer = Default::default();
                } else {
                    self.buffer = last_line.to_owned();
                }

                Ok(buf.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                let string = std::mem::take(&mut self.buffer);

                if string.trim().is_empty() {
                    return Ok(());
                }

                info!("{}", string.trim_end());

                Ok(())
            }
        }

        impl Drop for Logger {
            fn drop(&mut self) {
                use std::io::Write;

                self.flush().unwrap();
            }
        }

        self.pool.get().await?.interact(move |conn|
            embedded_migrations::run_with_output(conn, &mut Logger::default())
        ).await?
            .map_err(Into::into)
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
    #[error("{0}")]
    MigrationError(#[from] RunMigrationsError)
}
