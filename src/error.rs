use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum FileError {
    #[error("File does not exist: {0}")]
    FileDoesNotExist(String),
}

#[derive(Error, Debug)]
pub(crate) enum ResponseError {
    #[error("Missing payload placeholder: {0}")]
    MissingPlaceholder(String),
    #[error("Invalid payload template: {0}")]
    InvalidTemplate(String),

    #[error("Invalid parameters. Use -p or --profile to specify a profile.")]
    InvalidParameters,
}

impl From<strfmt::FmtError> for ResponseError {
    fn from(error: strfmt::FmtError) -> Self {
        match error {
            strfmt::FmtError::KeyError(message) => {
                let placeholder = message
                    .strip_prefix("Invalid key: ")
                    .unwrap_or(&message)
                    .to_owned();
                Self::MissingPlaceholder(placeholder)
            }
            error => Self::InvalidTemplate(error.to_string()),
        }
    }
}

impl IntoResponse for ResponseError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}
