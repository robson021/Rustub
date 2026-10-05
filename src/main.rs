mod config;
mod error;
mod file_utils;
mod model;

use crate::model::stub_response::{Method, Response as StubResponse};
use anyhow::Result;
use axum::response::Redirect;
use axum::routing::{delete, patch, post, put};
use axum::{
    Json, Router, body::Body, extract::Path, http::StatusCode, response::Response, routing::get,
};
use log::info;

const DEFAULT_STUB_CONFIG_PATH: &str = "./config/default";
const DEFAULT_SERVER_CONFIG_PATH: &str = "./config/default/server.yaml";

#[tokio::main]
async fn main() -> Result<()> {
    config::setup_logger();

    let server_config = config::read_server_config(DEFAULT_SERVER_CONFIG_PATH)?;
    let stubs_config = config::read_stubs_config(DEFAULT_STUB_CONFIG_PATH)?;

    info!("Server config: {:?}", server_config);
    info!("Stubs config: {:?}", stubs_config);

    let host = &format!("{}:{}", server_config.address, server_config.port);

    let mut app = Router::new()
        .route("/", get(Redirect::to("/hello")))
        .route("/hello", get(Json("Hello, World!")));

    for stub in stubs_config {
        let res = stub.response;
        let handler = move |_path_params: Option<Path<u32>>| {
            let res = res.clone();
            async move { build_response(res).await }
        };

        let req = stub.request;
        match req.method {
            Method::GET => app = app.route(&req.url, get(handler)),
            Method::POST => app = app.route(&req.url, post(handler)),
            Method::PUT => app = app.route(&req.url, put(handler)),
            Method::DELETE => app = app.route(&req.url, delete(handler)),
            Method::PATCH => app = app.route(&req.url, patch(handler)),
        }
    }

    let listener = tokio::net::TcpListener::bind(host).await?;

    info!("Running on: http://{}", host);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn build_response(res: StubResponse) -> Response {
    let mut builder = Response::builder().status(StatusCode::from_u16(res.status).unwrap());

    if let Some(headers) = res.headers {
        for (key, value) in headers {
            builder = builder.header(key, value);
        }
    }

    match res.body {
        Some(payload) => builder.body(Body::from(payload.to_string())).unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}
