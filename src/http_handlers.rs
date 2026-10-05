use crate::model::stub_response::Response as StubResponse;
use axum::http::StatusCode;
use axum::{body::Body, extract::Path, response::Response};

pub async fn build_response(res: StubResponse) -> Response {
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

pub async fn method_not_allowed(_path: Option<Path<u32>>) -> Response {
    Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::empty())
        .unwrap()
}
