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
pub(crate) struct Response {
    pub(crate) status: u16,
    pub(crate) body: Option<Value>,
    pub(crate) headers: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct Request {
    pub(crate) url: String,
    #[serde(deserialize_with = "method_from_str")]
    pub(crate) method: Method,
}

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct StubResponse {
    pub(crate) request: Request,
    pub(crate) response: Response,
}
