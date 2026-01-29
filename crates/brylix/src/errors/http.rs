//! HTTP-level error types for the Lambda handler.

use thiserror::Error;

/// Server-side errors (our fault)
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("could not serialize JSON")]
    Disconnect(#[from] serde_json::Error),

    #[error("error creating response")]
    Response(#[from] http::Error),
}

/// Client-side errors (their fault)
#[derive(Error, Debug)]
pub enum ClientError {
    #[error(transparent)]
    Query(#[from] async_graphql::ParseRequestError),

    #[error("Could not parse JSON body")]
    Json(#[from] serde_json::Error),

    #[error("Binary body must be encoded with UTF-8")]
    InvalidBinaryBody(#[from] std::str::Utf8Error),

    #[error("Only GET and POST methods are allowed")]
    MethodNotAllowed,
}

impl ServerError {
    /// Convert to Lambda error
    pub fn into_lambda_error(self) -> lambda_http::Error {
        lambda_http::Error::from(self.to_string())
    }
}

impl ClientError {
    /// Convert to Lambda error
    pub fn into_lambda_error(self) -> lambda_http::Error {
        lambda_http::Error::from(self.to_string())
    }
}
