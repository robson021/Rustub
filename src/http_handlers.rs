use crate::error::ResponseError;
use crate::model;
use crate::model::stub_response::Response as StubResponse;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    body::Body,
    extract::{Path, Query},
    response::Response,
};
use log::debug;
use std::collections::HashMap;
use std::future::{Ready, ready};
use std::sync::Arc;

pub(crate) type PathParams = Option<Path<HashMap<String, String>>>;
pub(crate) type QueryParams = Query<HashMap<String, String>>;

pub(crate) fn create_handler(
    url: String,
    res: model::stub_response::Response,
) -> impl Fn(PathParams, QueryParams) -> Ready<Response> + Clone {
    let res = Arc::new(res);
    move |path_params, query_map| {
        debug!(
            "Url: {} | path params: {:?} | query: {:?}",
            url, path_params, query_map
        );
        ready(
            build_response(&res, path_params, query_map).unwrap_or_else(|error| match error
                .downcast::<ResponseError>(
            ) {
                Ok(response_error) => response_error.into_response(),
                Err(error) => {
                    log::error!("Failed to build stub response: {error:#}");
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }),
        )
    }
}

fn build_response(
    res: &StubResponse,
    path: PathParams,
    query: QueryParams,
) -> anyhow::Result<Response> {
    let mut builder = Response::builder().status(StatusCode::from_u16(res.status)?);

    if let Some(headers) = &res.headers {
        for (key, value) in headers {
            builder = builder.header(key.as_str(), value.as_str());
        }
    }

    match &res.body {
        Some(payload) => {
            let mut params = HashMap::new();
            if let Some(Path(path_params)) = path {
                params.extend(path_params);
            }
            params.extend(query.0);

            let payload = substitute_params(payload, &params)?;
            Ok(builder.body(Body::from(payload.to_string()))?)
        }
        None => Ok(builder.body(Body::empty())?),
    }
}

fn substitute_params(
    value: &serde_json::Value,
    params: &HashMap<String, String>,
) -> anyhow::Result<serde_json::Value> {
    match value {
        serde_json::Value::String(text) if !text.contains('{') => {
            Ok(serde_json::Value::String(text.clone()))
        }
        serde_json::Value::String(text) => Ok(serde_json::Value::String(
            strfmt::strfmt(text, params).map_err(ResponseError::from)?,
        )),
        serde_json::Value::Array(items) => items
            .iter()
            .map(|item| substitute_params(item, params))
            .collect::<anyhow::Result<Vec<_>>>()
            .map(serde_json::Value::Array),
        serde_json::Value::Object(object) => object
            .into_iter()
            .map(|(key, value)| Ok((key.clone(), substitute_params(value, params)?)))
            .collect::<anyhow::Result<_>>()
            .map(serde_json::Value::Object),
        value => Ok(value.clone()),
    }
}

pub(crate) async fn method_not_allowed(_path: Option<Path<String>>) -> Response {
    Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::empty())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use quickcheck::quickcheck;
    use serde_json::json;
    use std::collections::HashMap;

    quickcheck! {
        #[test]
        fn substitutes_arbitrary_values_recursively(value: String, count: u8) -> bool {
            let params = HashMap::from([("value".to_string(), value.clone())]);
            let payload = serde_json::Value::Array(
                (0..count)
                    .map(|_| json!({"nested": ["prefix-{value}", {"value": "{value}"}]}))
                    .collect(),
            );
            let expected = serde_json::Value::Array(
                (0..count)
                    .map(|_| json!({"nested": [format!("prefix-{value}"), {"value": value}]}))
                    .collect(),
            );

            substitute_params(&payload, &params).is_ok_and(|result| result == expected)
        }
    }

    #[test]
    fn substitutes_params_in_nested_objects_and_arrays() {
        let params = HashMap::from([
            ("id".to_string(), "42".to_string()),
            ("tenant".to_string(), "acme".to_string()),
            ("role".to_string(), "admin".to_string()),
            ("active".to_string(), "true".to_string()),
        ]);
        let payload = json!({
            "user": {
                "id": "{id}",
                "username": "user_{id}",
                "tenant": "{tenant}",
                "roles": ["{role}", "auditor"],
                "metadata": [
                    {"active": "{active}"},
                    {"description": "account for {tenant}/{id}"}
                ]
            },
            "count": 2,
            "enabled": true,
            "nothing": null
        });

        let result = substitute_params(&payload, &params).unwrap();

        assert_eq!(
            result,
            json!({
                "user": {
                    "id": "42",
                    "username": "user_42",
                    "tenant": "acme",
                    "roles": ["admin", "auditor"],
                    "metadata": [
                        {"active": "true"},
                        {"description": "account for acme/42"}
                    ]
                },
                "count": 2,
                "enabled": true,
                "nothing": null
            })
        );
    }

    #[test]
    fn reports_missing_placeholder_inside_nested_array() {
        let payload = json!({"items": [{"id": "{missing}"}]});

        let error = substitute_params(&payload, &HashMap::new()).unwrap_err();

        assert!(matches!(
            error.downcast_ref::<ResponseError>(),
            Some(ResponseError::MissingPlaceholder(placeholder)) if placeholder == "missing"
        ));
    }

    #[test]
    fn plain_text_body_and_headers_applied() {
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

        let resp = build_response(&res, None, Query(HashMap::new())).unwrap();
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

    #[test]
    fn complex_json_body_is_serialized_and_headers_set() {
        let payload = json!({
            "user": {"id": 1, "name": "Alice"},
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

        let resp = build_response(&res, None, Query(HashMap::new())).unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let ct = resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(ct, ct_val);
        let expected = serde_json::to_string(&payload).unwrap();
        assert!(expected.contains("\"name\":\"Alice\""));
        assert!(expected.contains("\"roles\""));
    }

    #[test]
    fn no_body_returns_empty_response_and_respects_status() {
        let res = StubResponse {
            status: 204,
            body: None,
            headers: None,
        };

        let resp = build_response(&res, None, Query(HashMap::new())).unwrap();
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
                "unmatched": "no placeholder here"
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

        let response = build_response(&res, path, query).unwrap();
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
                "unmatched": "no placeholder here"
            })
        );
    }

    #[test]
    fn missing_placeholder_returns_bad_request() {
        let res = StubResponse {
            status: 200,
            body: Some(json!({"id": "{missing}"})),
            headers: None,
        };

        let error = build_response(&res, None, Query(HashMap::new())).unwrap_err();
        assert!(matches!(
            error.downcast_ref::<ResponseError>(),
            Some(ResponseError::MissingPlaceholder(placeholder)) if placeholder == "missing"
        ));
        let response = error.downcast::<ResponseError>().unwrap().into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
