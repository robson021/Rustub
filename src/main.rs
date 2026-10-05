mod config;
mod error;
mod file_utils;
mod http_handlers;
mod model;

use crate::http_handlers::{build_response, method_not_allowed};
use anyhow::Result;
use axum::http::Method;
use axum::response::Redirect;
use axum::routing::{any, delete, patch, post, put};
use axum::{Json, Router, extract::Path, routing::get};
use log::{debug, info};

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
        let req = stub.request;
        let res = stub.response;
        let url = req.url.clone();

        let handler = move |path_params: Option<Path<String>>| {
            debug!("Path params for {}: {:?}", url, path_params);
            let res = res.clone();
            async move { build_response(res, path_params, None).await }
        };

        match req.method {
            Method::GET => app = app.route(&req.url, get(handler)),
            Method::POST => app = app.route(&req.url, post(handler)),
            Method::PUT => app = app.route(&req.url, put(handler)),
            Method::DELETE => app = app.route(&req.url, delete(handler)),
            Method::PATCH => app = app.route(&req.url, patch(handler)),
            _ => app = app.route(&req.url, any(method_not_allowed)),
        }
    }

    let listener = tokio::net::TcpListener::bind(host).await?;

    info!("Running on: http://{}", host);
    axum::serve(listener, app).await?;
    Ok(())
}
