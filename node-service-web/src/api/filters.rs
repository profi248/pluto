use warp::Filter;

use pluto_network::client::Client;

use crate::api::routes;
use crate::KeysShared;

pub fn status(client: Client) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "status")
        .and(warp::get())
        .and(with_client(client))
        .and_then(routes::status::get_status)
}

pub fn setup(client: Client, keys: KeysShared) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "setup")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024))
        .and(warp::body::json())
        .and(with_client(client))
        .and(with_keys(keys))
        .and_then(routes::setup::setup)
}

fn with_client(client: Client) -> impl Filter<Extract=(Client, ), Error=std::convert::Infallible> + Clone {
    warp::any().map(move || (client.clone()))
}

fn with_keys(keys: KeysShared) -> impl Filter<Extract=(KeysShared, ), Error=std::convert::Infallible> + Clone {
    warp::any().map(move || (keys.clone()))
}
