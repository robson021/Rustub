mod config;
mod error;
mod file_utils;

use crate::config::ServerConfig;
use axum::response::Redirect;
use axum::routing::get;
use axum::{Json, Router};
use log::info;

#[tokio::main]
async fn main() {
    config::setup_logger();
    let cfg: ServerConfig = config::read_config("./config/default/server.yaml").unwrap();
    info!("Server config: {:?}", cfg);

    let host = format!("{}:{}", cfg.address, cfg.port);

    let app = Router::new()
        .route("/", get(Redirect::to("/hello")))
        .route("/hello", get(Json("Hello, World!")));

    let listener = tokio::net::TcpListener::bind(&host).await.unwrap();

    info!("Running on: http://{}", host);
    axum::serve(listener, app).await.unwrap();
}
