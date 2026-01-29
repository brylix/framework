//! Lambda HTTP handler for GraphQL APIs.
//!
//! Provides utilities for handling GraphQL requests in AWS Lambda,
//! including CORS, playground, multipart uploads, and tenant routing.
//!
//! # Usage
//!
//! ```rust
//! use brylix::handler::{handle_request, graphql_error};
//! use lambda_http::{run, Error};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     run(|req| handle_request(req)).await
//! }
//! ```

mod cors;
mod multipart;
mod router;

pub use cors::{cors_headers, cors_preflight};
pub use multipart::parse_multipart;
pub use router::{extract_tenant, extract_playground_tenant};

use async_graphql::{
    http::GraphQLPlaygroundConfig, http::playground_source, Error as GqlError, Pos,
    Request as GraphQlRequest, Response as GraphQlResponse,
};
use http::StatusCode;
use lambda_http::{Body, Error, Request, RequestExt, Response};

use crate::errors::ServerError;

/// Create a GraphQL error response from a message.
pub fn graphql_error(message: impl std::fmt::Display) -> String {
    let msg = format!("{}", message);
    let gql_err: GqlError = crate::errors::gql_from_message(msg);
    let server_err = gql_err.into_server_error(Pos::default());
    let response = GraphQlResponse::from_errors(vec![server_err]);
    serde_json::to_string(&response).unwrap_or_else(|_| {
        r#"{"errors":[{"message":"Internal serialization error"}]}"#.to_string()
    })
}

/// Create an HTTP error response.
pub fn error_response(status: StatusCode, body: String) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", "*")
        .body(Body::Text(body))?)
}

/// Create a GraphQL Playground HTML response.
pub fn playground_response(api_endpoint: &str) -> Result<Response<Body>, Error> {
    let playground_config = GraphQLPlaygroundConfig::new(api_endpoint);
    let html = playground_source(playground_config);
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .header("Access-Control-Allow-Origin", "*")
        .body(Body::Text(html))
        .map_err(|e| Error::from(format!("Playground response error: {}", e)))
}

/// Parse a GraphQL request from a POST body.
pub async fn graphql_request_from_post(request: Request) -> anyhow::Result<GraphQlRequest> {
    let content_type = request
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if content_type.starts_with("multipart/form-data") {
        parse_multipart(request).await
    } else {
        let body = request.body();
        let body_str = std::str::from_utf8(body).map_err(|e| anyhow::anyhow!("{}", e))?;
        serde_json::from_str(body_str).map_err(|e| anyhow::anyhow!("{}", e))
    }
}

/// Parse a GraphQL request from GET query parameters.
pub async fn graphql_request_from_get(request: Request) -> anyhow::Result<GraphQlRequest> {
    let query_params = request.query_string_parameters();
    let query = query_params
        .first("query")
        .ok_or_else(|| anyhow::anyhow!("Missing query parameter"))?;

    let variables = query_params
        .first("variables")
        .and_then(|v| serde_json::from_str(v).ok());

    let operation_name = query_params.first("operationName").map(|s| s.to_string());

    Ok(GraphQlRequest::new(query.to_string())
        .variables(variables.unwrap_or_default())
        .operation_name(operation_name.unwrap_or_default()))
}

/// Build a successful GraphQL response.
pub fn graphql_response(response: GraphQlResponse) -> Result<Response<Body>, Error> {
    let response_body = serde_json::to_string(&response).map_err(ServerError::from)?;
    Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .body(Body::Text(response_body))
        .map_err(ServerError::from)
        .map_err(Error::from)
}
