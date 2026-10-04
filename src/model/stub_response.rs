use serde::Deserialize;
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
#[serde(untagged)]
pub enum ResponseBody {
    Json(HashMap<String, serde_json::Value>),
    Text(String),
}

#[derive(Deserialize, Debug)]
pub struct Response {
    pub status: u16,
    pub body: Option<ResponseBody>,
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
