use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Response {
    status: u16,
    body: String,
    headers: Vec<(String, String)>,
}

#[derive(Debug, Deserialize)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
    Trace,
}

#[derive(Debug, Deserialize)]
pub struct Request {
    url: String,
    method: Method,
}
#[derive(Debug, Deserialize)]
pub struct StubResponse {
    request: Request,
    response: Response,
}
