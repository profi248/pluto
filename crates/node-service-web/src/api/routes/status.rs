use std::convert::Infallible;

use warp::{ reply, http::StatusCode };
use serde::Serialize;

use pluto_network::Client;
use pluto_node::db::Database;

#[derive(Serialize)]
struct Status {
    setup_complete: bool,
    connected: bool,
}

#[derive(Serialize)]
struct Error {
    error: String
}

pub async fn get_status(client: Client) -> Result<impl warp::Reply, Infallible> {
    let setup_done = Database::new().get_initial_setup_done();
    if setup_done.is_none() {
        return Ok(reply::with_status(reply::json(&Error { error: "Database error".to_string() }),
                                     StatusCode::INTERNAL_SERVER_ERROR));
    }

    Ok(reply::with_status(reply::json(&Status {
        setup_complete: setup_done.unwrap(),
        connected: client.is_connected(),
    }), StatusCode::OK))
}
