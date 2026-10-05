use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub body: Option<Value>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Request {
    pub url: String,
    pub method: Method,
}
#[derive(Deserialize, Clone, Debug)]
pub struct StubResponse {
    pub request: Request,
    pub response: Response,
}
