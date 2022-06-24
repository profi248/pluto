mod filters;
mod routes;

use rust_embed::RustEmbed;
use warp::{ http::header::HeaderValue, path::Tail, reply::Response, Filter, Rejection, Reply };

use pluto_network::{ client::Client, key::Keys };

#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct Asset;

pub async fn run(addr: impl Into<std::net::SocketAddr>, client: &Client, keys: &Keys) {
    let index_html = warp::path::end().and_then(serve_static_index);
    let dist = warp::path::tail().and_then(serve_static);

    let client = client.clone();
    let keys = keys.clone();

    let routes =
        index_html
        .or(dist)

        .or(filters::status(client, keys));

    warp::serve(routes).run(addr).await;
}

async fn serve_static_index() -> Result<impl Reply, Rejection> {
    serve_static_impl("index.html")
}

async fn serve_static(path: Tail) -> Result<impl Reply, Rejection> {
    serve_static_impl(path.as_str())
}

fn serve_static_impl(path: &str) -> Result<impl Reply, Rejection> {
    let asset = Asset::get(path).ok_or_else(warp::reject::not_found)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut res = Response::new(asset.data.into());
    res.headers_mut().insert("content-type", HeaderValue::from_str(mime.as_ref()).unwrap());
    Ok(res)
}
