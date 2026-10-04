mod config;
mod error;
mod file_utils;
mod model;

use crate::model::stub_response::Method;
use anyhow::Result;
use axum::response::Redirect;
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
        let req = stub.request;
        let res = stub.response;

        let handler = move |path_params: Option<Path<u32>>| {
            async move {
                let entity_id = match path_params {
                    Some(Path(id)) => id.to_string(),
                    None => "No ID".to_string(),
                };
                let payload = format!(r#"{{"entity_id": "{entity_id}"}}"#);

                // todo add headers
                Response::builder()
                    .status(StatusCode::from_u16(res.status).unwrap())
                    .header("Content-Type", "application/json")
                    .body(Body::from(payload))
                    .unwrap()
            }
        };
        // todo: add custom method mapping
        match req.method {
            Method::GET => {}
            Method::POST => {}
            Method::PUT => {}
            Method::DELETE => {}
            Method::PATCH => {}
        }
        app = app.route(&req.url, get(handler));
    }

    let listener = tokio::net::TcpListener::bind(host).await?;

    info!("Running on: http://{}", host);
    axum::serve(listener, app).await?;
    Ok(())
}
