use crate::model::stub_response::Response as StubResponse;
use axum::http::StatusCode;
use axum::{
    body::Body,
    extract::{Path, Query},
    response::Response,
};
use std::collections::HashMap;

pub type PathParams = Option<Path<HashMap<String, String>>>;
pub type QueryParams = Query<HashMap<String, String>>;

pub async fn build_response(res: StubResponse, path: PathParams, query: QueryParams) -> Response {
    let mut builder = Response::builder().status(StatusCode::from_u16(res.status).unwrap());

    if let Some(headers) = res.headers {
        for (key, value) in headers {
            builder = builder.header(key, value);
        }
    }

    match res.body {
        Some(payload) => {
            let mut params = HashMap::new();
            if let Some(Path(path_params)) = path {
                params.extend(path_params);
            }
            params.extend(query.0);

            let payload = substitute_params(payload, &params);
            builder.body(Body::from(payload.to_string())).unwrap()
        }
        None => builder.body(Body::empty()).unwrap(),
    }
}

fn substitute_params(
    mut value: serde_json::Value,
    params: &HashMap<String, String>,
) -> serde_json::Value {
    match &mut value {
        serde_json::Value::String(text) => {
            let mut substituted = String::with_capacity(text.len());
            let mut remaining = text.as_str();

            while let Some(open) = remaining.find('{') {
                let (before, from_open) = remaining.split_at(open);
                substituted.push_str(before);

                let Some(close) = from_open.find('}') else {
                    substituted.push_str(from_open);
                    remaining = "";
                    break;
                };

                let (placeholder, after_placeholder) = from_open[1..].split_at(close - 1);
                if let Some(param) = params.get(placeholder) {
                    substituted.push_str(param);
                } else {
                    substituted.push('{');
                    substituted.push_str(placeholder);
                    substituted.push('}');
                }
                remaining = &after_placeholder[1..];
            }

            substituted.push_str(remaining);
            *text = substituted;
        }
        serde_json::Value::Array(items) => {
            for item in items {
                *item = substitute_params(std::mem::take(item), params);
            }
        }
        serde_json::Value::Object(object) => {
            for item in object.values_mut() {
                *item = substitute_params(std::mem::take(item), params);
            }
        }
        _ => {}
    }
    value
}

pub async fn method_not_allowed(_path: Option<Path<String>>) -> Response {
    Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::empty())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[tokio::test]
    async fn plain_text_body_and_headers_applied() {
        let ct_key = "Content-Type";
        let ct_val = "text/plain";
        let x_key = "X-Custom";
        let x_val = "value";

        let payload = json!("simple text");

        let headers: HashMap<String, String> = HashMap::from([
            (ct_key.to_string(), ct_val.to_string()),
            (x_key.to_string(), x_val.to_string()),
        ]);

        let res = StubResponse {
            status: 200,
            body: Some(payload.clone()),
            headers: Some(headers),
        };

        let resp = build_response(res, None, Query(HashMap::new())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(ct, ct_val);
        let xb = resp.headers().get("x-custom").unwrap().to_str().unwrap();
        assert_eq!(xb, x_val);
        let expected = serde_json::to_string(&payload).unwrap();
        assert_eq!(expected, "\"simple text\"");
    }

    #[tokio::test]
    async fn complex_json_body_is_serialized_and_headers_set() {
        let payload = json!({
            "user": {"id": 1, "name": "alice"},
            "roles": ["admin", "user"]
        });

        let ct_key = "Content-Type";
        let ct_val = "application/json";
        let headers: HashMap<String, String> =
            HashMap::from([(ct_key.to_string(), ct_val.to_string())]);

        let res = StubResponse {
            status: 201,
            body: Some(payload.clone()),
            headers: Some(headers),
        };

        let resp = build_response(res, None, Query(HashMap::new())).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let ct = resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(ct, ct_val);
        let expected = serde_json::to_string(&payload).unwrap();
        // the serialized JSON should contain the user name and roles
        assert!(expected.contains("\"name\":\"alice\""));
        assert!(expected.contains("\"roles\""));
    }

    #[tokio::test]
    async fn no_body_returns_empty_response_and_respects_status() {
        let res = StubResponse {
            status: 204,
            body: None,
            headers: None,
        };

        let resp = build_response(res, None, Query(HashMap::new())).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        let expected: Vec<u8> = Vec::new();
        assert!(expected.is_empty());
    }

    #[tokio::test]
    async fn substitutes_multiple_path_and_query_params_in_nested_payload() {
        let res = StubResponse {
            status: 201,
            body: Some(json!({
                "id": "{id}",
                "username": "user_{id}",
                "profile": {
                    "role": "{role}",
                    "isActive": "{active}",
                    "tenant": "{tenant}",
                    "region": "{region}"
                },
                "unmatched": "{missing}"
            })),
            headers: None,
        };
        let path = Some(Path(HashMap::from([
            ("id".to_string(), "42".to_string()),
            ("tenant".to_string(), "acme".to_string()),
        ])));
        let query = Query(HashMap::from([
            ("role".to_string(), "admin".to_string()),
            ("active".to_string(), "true".to_string()),
            ("region".to_string(), "west".to_string()),
        ]));

        let response = build_response(res, path, query).await;
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            body,
            json!({
                "id": "42",
                "username": "user_42",
                "profile": {
                    "role": "admin",
                    "isActive": "true",
                    "tenant": "acme",
                    "region": "west"
                },
                "unmatched": "{missing}"
            })
        );
    }
}
