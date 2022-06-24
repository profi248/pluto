use std::convert::Infallible;

use warp::{ reply, http::StatusCode };
use serde::Serialize;

use pluto_node::db::Database;

#[derive(Serialize)]
struct Status {
    setup_complete: bool
}

#[derive(Serialize)]
struct Error {
    error: String
}

pub async fn get_status() -> Result<impl warp::Reply, Infallible> {
    let setup_done = Database::get_initial_setup_done();
    if setup_done.is_none() {
        return Ok(reply::with_status(reply::json(&Error { error: "Database error".to_string() }),
                                     StatusCode::INTERNAL_SERVER_ERROR));
    }

    Ok(reply::with_status(reply::json(&Status {
        setup_complete: setup_done.unwrap()
    }), StatusCode::INTERNAL_SERVER_ERROR))
}
