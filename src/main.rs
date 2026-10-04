mod config;
mod error;
mod file_utils;
mod model;

use axum::response::Redirect;
use axum::routing::get;
use axum::{Json, Router};
use log::{error, info};

const DEFAULT_CONFIG_PATH: &str = "./config/default/server.yaml";

#[tokio::main]
async fn main() {
    config::setup_logger();
    let config = match config::read_config(DEFAULT_CONFIG_PATH) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };
    info!("Server config: {:?}", config);

    let host = format!("{}:{}", config.address, config.port);

    let app = Router::new()
        .route("/", get(Redirect::to("/hello")))
        .route("/hello", get(Json("Hello, World!")));

    let listener = tokio::net::TcpListener::bind(&host).await.unwrap();

    info!("Running on: http://{}", host);
    axum::serve(listener, app).await.unwrap();
}
