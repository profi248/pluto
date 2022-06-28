mod schema;
pub mod models;

use diesel::{ Connection, QueryResult, RunQueryDsl, SqliteConnection, sql_query };

embed_migrations!();

no_arg_sql_function!(
    last_insert_rowid,
    diesel::sql_types::Integer,
    "Represents the SQL last_insert_row() function"
);

pub struct Database {
    conn: SqliteConnection
}

impl Database {
    pub fn new() -> Self {
        Self {
            conn: Self::connect()
        }
    }

    fn connect() -> SqliteConnection {
        let file = crate::utils::get_db_file_path();
        SqliteConnection::establish(file.as_str())
            .unwrap_or_else(|_| panic!("Error opening database file at {file}"))
    }

    pub fn check_connection(&self) -> bool {
        sql_query("select 1;").execute(&self.conn).is_ok()
    }

    pub fn run_migrations(&self) -> Option<()> {
        embedded_migrations::run(&self.conn).ok()
    }

    pub fn begin_transaction(&self) -> QueryResult<()> {
        sql_query("BEGIN TRANSACTION;").execute(&self.conn)?;
        Ok(())
    }

    pub fn commit_transaction(&self) -> QueryResult<()> {
        sql_query("COMMIT TRANSACTION;").execute(&self.conn)?;
        Ok(())
    }

    pub fn rollback_transaction(&self) -> QueryResult<()> {
        sql_query("ROLLBACK TRANSACTION;").execute(&self.conn)?;
        Ok(())
    }

    pub fn set_initial_setup_done(&self, value: bool) -> Option<()> {
        Self::set_by_key(&self, "setup_completed".to_owned(),
                         vec![value as u8]).ok()
    }

    pub fn get_initial_setup_done(&self) -> Option<bool> {
        match Self::get_by_key(&self, "setup_completed".to_owned()) {
            Ok(value) => {
                let value = value.unwrap_or(vec![0]);
                Some(value.len() == 1 && value[0] == 0x1)
            }
            Err(_) => None,
        }
    }
}
