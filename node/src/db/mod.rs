mod schema;
mod models;

use diesel::{ Connection, RunQueryDsl, SqliteConnection };

embed_migrations!();

pub struct Database;

impl Database {
    pub fn connect() -> SqliteConnection {
        let file = crate::utils::get_db_file_path();
        SqliteConnection::establish(file.as_str())
            .unwrap_or_else(|_| panic!("Error opening database file at {file}"))
    }

    pub fn check_connection(&self) -> bool {
        diesel::sql_query("select 1;").execute(&Self::connect()).is_ok()
    }

    pub fn run_migrations() -> Option<()> {
        embedded_migrations::run(&Self::connect()).ok()
    }
}
