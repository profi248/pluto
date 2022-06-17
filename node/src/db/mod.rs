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

    pub fn set_initial_setup_done(value: bool) -> Option<()> {
        Self::set_by_key("setup_completed".to_owned(),
                         vec![value as u8]).ok()
    }

    pub fn get_initial_setup_done() -> Option<bool> {
        match Self::get_by_key("setup_completed".to_owned()) {
            Ok(value) => {
                let value = value.unwrap_or(vec![0]);
                Some(value.len() == 1 && value[0] == 0x1)
            }
            Err(_) => None,
        }
    }
}
