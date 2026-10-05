use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

#[derive(Deserialize, Debug)]
pub struct Response {
    pub status: u16,
    pub body: Option<Value>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
pub struct Request {
    pub url: String,
    pub method: Method,
}
#[derive(Deserialize, Debug)]
pub struct StubResponse {
    pub request: Request,
    pub response: Response,
}
