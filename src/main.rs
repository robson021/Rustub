mod config;

use axum::response::Redirect;
use axum::routing::get;
use axum::{Json, Router};
use log::info;

#[tokio::main]
async fn main() {
    config::setup_logger();

    let address = "127.0.0.1";
    let port = 8080;
    let host = format!("{}:{}", address, port);

    let app = Router::new()
        .route("/", get(Redirect::to("/hello")))
        .route("/hello", get(Json("Hello, World!")));

    let listener = tokio::net::TcpListener::bind(&host).await.unwrap();

    info!("Running on: http://{}", host);
    axum::serve(listener, app).await.unwrap();
}
