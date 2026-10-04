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
enum ResponseBody {
    Json(HashMap<String, serde_json::Value>),
    Text(String),
}

#[derive(Deserialize, Debug)]
pub struct Response {
    status: u16,
    body: Option<ResponseBody>,
    headers: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
pub struct Request {
    url: String,
    method: Method,
}
#[derive(Deserialize, Debug)]
pub struct StubResponse {
    request: Request,
    response: Response,
}
