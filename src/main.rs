mod config;
mod error;
mod file_utils;
mod http_handlers;
mod model;

use crate::http_handlers::{create_handler, method_not_allowed};
use crate::model::server_config::resolve_config_path;
use crate::model::stub_response::StubResponse;
use anyhow::Result;
use axum::http::Method;
use axum::response::Redirect;
use axum::routing::{any, delete, patch, post, put};
use axum::{Json, Router, routing::get};
use log::{debug, info};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    config::setup_logger();

    let args: Vec<String> = env::args().skip(1).collect();
    debug!("Cmd args: {:?}", args);

    let cfg_path = resolve_config_path(&args)?;
    let server_cfg_path = format!("{cfg_path}/server.yaml");

    let server_config = config::read_server_config(&server_cfg_path)?;
    let stubs_config = config::read_stubs_config(&cfg_path)?;

    info!("Server config: {:?}", server_config);
    info!("Stubs config: {:?}", stubs_config);

    let host = &format!("{}:{}", server_config.address, server_config.port);
    let listener = tokio::net::TcpListener::bind(host).await?;
    let app = build_routes_for_stubs(stubs_config);

    info!("Running on: http://{}", host);
    axum::serve(listener, app).await?;
    Ok(())
}

fn build_routes_for_stubs(stubs_config: Vec<StubResponse>) -> Router {
    let mut app = Router::new()
        .route("/", get(Redirect::to("/hello")))
        .route("/hello", get(Json("Hello, World!")));

    for stub in stubs_config {
        let req = stub.request;
        let res = stub.response;
        let url = req.url.clone();
        let route_path = req.url.split('?').next().unwrap_or(&req.url);

        let handler = create_handler(url, res);

        match req.method {
            Method::GET => app = app.route(route_path, get(handler)),
            Method::POST => app = app.route(route_path, post(handler)),
            Method::PUT => app = app.route(route_path, put(handler)),
            Method::DELETE => app = app.route(route_path, delete(handler)),
            Method::PATCH => app = app.route(route_path, patch(handler)),
            _ => app = app.route(route_path, any(method_not_allowed)),
        }
    }
    app
}
