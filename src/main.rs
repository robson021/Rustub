mod config;
mod error;
mod file_utils;
mod http_handlers;
mod model;

use crate::http_handlers::{PathParams, QueryParams, build_response, method_not_allowed};
use crate::model::stub_response::StubResponse;
use anyhow::Result;
use axum::http::Method;
use axum::response::Redirect;
use axum::routing::{any, delete, patch, post, put};
use axum::{Json, Router, routing::get};
use log::{debug, info};
use std::{future::Future, pin::Pin};

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

        let handler = create_handler(url, res);

        match req.method {
            Method::GET => app = app.route(&req.url, get(handler)),
            Method::POST => app = app.route(&req.url, post(handler)),
            Method::PUT => app = app.route(&req.url, put(handler)),
            Method::DELETE => app = app.route(&req.url, delete(handler)),
            Method::PATCH => app = app.route(&req.url, patch(handler)),
            _ => app = app.route(&req.url, any(method_not_allowed)),
        }
    }
    app
}

fn create_handler(
    url: String,
    res: model::stub_response::Response,
) -> impl Fn(PathParams, QueryParams) -> Pin<Box<dyn Future<Output = axum::response::Response> + Send>>
+ Clone {
    move |path_params, query_map| {
        debug!(
            "Url: {} | path params: {:?} | query: {:?}",
            url, path_params, query_map
        );
        let res = res.clone();
        Box::pin(async move { build_response(res, path_params, query_map).await })
    }
}
