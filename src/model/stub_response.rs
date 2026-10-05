use axum::http::Method;
use serde::Deserialize;
use serde::de::{self, Deserializer};
use serde_json::Value;
use std::collections::HashMap;

fn method_from_str<'de, D>(deserializer: D) -> Result<Method, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse()
        .map_err(|e| de::Error::custom(format!("invalid HTTP method: {}", e)))
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
    #[serde(deserialize_with = "method_from_str")]
    pub method: Method,
}

#[derive(Deserialize, Clone, Debug)]
pub struct StubResponse {
    pub request: Request,
    pub response: Response,
}
