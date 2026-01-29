//! CORS handling for Lambda requests.

use http::StatusCode;
use lambda_http::{Body, Error, Response};

/// CORS headers for responses.
pub fn cors_headers() -> [(&'static str, &'static str); 3] {
    [
        ("Access-Control-Allow-Origin", "*"),
        ("Access-Control-Allow-Methods", "GET,POST,OPTIONS"),
        (
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        ),
    ]
}

/// Handle CORS preflight OPTIONS request.
pub fn cors_preflight() -> Result<Response<Body>, Error> {
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET,POST,OPTIONS")
        .header(
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization",
        )
        .header("Access-Control-Max-Age", "86400")
        .body(Body::Empty)
        .map_err(|e| Error::from(format!("CORS response error: {}", e)))
}

/// Check if a request is a CORS preflight request.
pub fn is_preflight(method: &http::Method, path: &str) -> bool {
    method == http::Method::OPTIONS && (path == "/api" || path.starts_with("/api/"))
}
