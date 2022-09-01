mod filters;
mod routes;

use rust_embed::RustEmbed;
use warp::{ http::header::HeaderValue, path::Tail, reply::Response, Filter, Rejection, Reply };

use pluto_network::client::Client;
use crate::KeysShared;

#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct Asset;

pub async fn run(addr: impl Into<std::net::SocketAddr>, client: &Client, keys: &KeysShared) {
    let index_html = warp::path::end().and_then(serve_static_index);
    let dist = warp::path::tail().and_then(serve_static);

    let client = client.clone();
    let keys = keys.clone();

    let routes =
        index_html
        .or(filters::status(client.clone()))
        .or(filters::setup(client.clone(), keys.clone()))
        .or(filters::get_jobs())
        .or(filters::create_job(client.clone(), keys.clone()))
        .or(filters::update_job(client.clone(), keys.clone()))
        .or(filters::delete_job(client.clone(), keys.clone()))
        .or(filters::create_job_path())
        .or(filters::update_job_path())
        .or(filters::delete_job_path())
        .or(filters::get_nodes())

        .or(dist);

    warp::serve(routes).run(addr).await;
}

async fn serve_static_index() -> Result<impl Reply, Rejection> {
    serve_static_impl("index.html")
}

async fn serve_static(path: Tail) -> Result<impl Reply, Rejection> {
    serve_static_impl(path.as_str())
}

fn serve_static_impl(path: &str) -> Result<impl Reply, Rejection> {
    // if file is not found, pass it on to JavaScript frontend for routing
    let mime;
    let asset = match Asset::get(path) {
        Some(asset) => {
            mime = mime_guess::from_path(path).first_or_octet_stream();
            asset
        },
        None => {
            mime = mime_guess::from_ext("html").first().unwrap();
            Asset::get("index.html").unwrap()
        }
    };

    let mut res = Response::new(asset.data.into());
    res.headers_mut().insert("content-type", HeaderValue::from_str(mime.as_ref()).unwrap());
    Ok(res)
}
