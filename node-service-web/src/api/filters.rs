use warp::Filter;

use pluto_network::{ client::Client, key::Keys };

use crate::api::routes;

pub fn status(client: Client, keys: Keys) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "status")
        .and(warp::get())
        .and(with_client(client))
        .and_then(routes::status::get_status)
}

fn with_client(client: Client) -> impl Filter<Extract=(Client, ), Error=std::convert::Infallible> + Clone {
    warp::any().map(move || (client.clone()))
}

fn with_keys(keys: Keys) -> impl Filter<Extract=(Keys, ), Error=std::convert::Infallible> + Clone {
    warp::any().map(move || (keys.clone()))
}
