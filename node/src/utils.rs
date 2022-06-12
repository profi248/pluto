use std::path::PathBuf;

pub const PLUTO_DIR: &str = "pluto";
pub const LOG_FILE: &str = "log.txt";
pub const DB_FILE: &str = "pluto.db";

pub fn get_pluto_dir() -> PathBuf {
    let mut path = dirs::config_dir().unwrap();
    path.push(PLUTO_DIR);

    path
}

pub fn get_db_file_path() -> String {
    let mut path = get_pluto_dir();
    path.push(DB_FILE);

    path.to_str().unwrap().to_owned()
}

pub fn get_log_file_path() -> String {
    let mut path = get_pluto_dir();
    path.push(LOG_FILE);

    path.to_str().unwrap().to_owned()
}

pub fn setup_dirs() {
    let mut path = get_pluto_dir();

    if !path.exists() { std::fs::create_dir(&path).unwrap(); }
    if !path.is_dir() { panic!("{} is a file.", PLUTO_DIR); }
}
