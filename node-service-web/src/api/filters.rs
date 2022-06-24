use warp::Filter;
use crate::api::routes;

pub fn status() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "status")
        .and(warp::get())
        .and_then(routes::status::get_status)
}
