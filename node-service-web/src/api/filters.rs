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

pub fn get_jobs(client: Client, keys: KeysShared) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "backup_jobs")
        .and(warp::get())
        .and(with_client(client))
        .and(with_keys(keys))
        .and_then(routes::backup_job::get_jobs)
}

pub fn create_job(client: Client, keys: KeysShared) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "backup_jobs")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024))
        .and(warp::body::json())
        .and(with_client(client))
        .and(with_keys(keys))
        .and_then(routes::backup_job::create_job)
}

pub fn update_job(client: Client, keys: KeysShared) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "backup_jobs" / i32)
        .and(warp::put())
        .and(warp::body::content_length_limit(1024))
        .and(warp::body::json())
        .and(with_client(client))
        .and(with_keys(keys))
        .and_then(routes::backup_job::update_job)
}

pub fn delete_job(client: Client, keys: KeysShared) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "backup_jobs" / i32)
        .and(warp::delete())
        .and(with_client(client))
        .and(with_keys(keys))
        .and_then(routes::backup_job::delete_job)
}

fn with_client(client: Client) -> impl Filter<Extract=(Client, ), Error=std::convert::Infallible> + Clone {
    warp::any().map(move || (client.clone()))
}

fn with_keys(keys: KeysShared) -> impl Filter<Extract=(KeysShared, ), Error=std::convert::Infallible> + Clone {
    warp::any().map(move || (keys.clone()))
}
