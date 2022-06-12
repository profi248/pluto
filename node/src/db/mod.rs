mod schema;
mod models;

use diesel::{ Connection, RunQueryDsl, SqliteConnection };

embed_migrations!();

pub enum DatabaseError {
    QueryFailed
}

pub struct Database;

impl Database {
    pub fn connect() -> SqliteConnection {
        let database_url = String::from("sqlite://") +
            crate::utils::get_db_file_path().as_str();

        SqliteConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error opening database file at {}", database_url))
    }

    pub fn check_connection(&self) -> bool {
        diesel::sql_query("select 1;").execute(&Self::connect()).is_ok()
    }
}
