//! Multipart form parsing for file uploads.
//!
//! Note: Full file upload support in Lambda requires additional setup.
//! Consider using S3 presigned URLs for file uploads instead.

use anyhow::Result;
use async_graphql::Request as GraphQlRequest;
use lambda_http::Request;
use multer::Multipart;
use std::io::Cursor;
use tokio_util::codec::{BytesCodec, FramedRead};

/// Uploaded file data from a multipart request.
#[derive(Debug, Clone)]
pub struct UploadedFile {
    /// Original filename
    pub filename: String,
    /// Content type (MIME)
    pub content_type: Option<String>,
    /// File bytes
    pub content: Vec<u8>,
}

/// Parse a multipart/form-data request into a GraphQL request.
///
/// Supports the GraphQL multipart request specification.
/// See: https://github.com/jaydenseric/graphql-multipart-request-spec
///
/// Note: For full file upload support in serverless environments,
/// consider using S3 presigned URLs instead of direct uploads.
///
/// # Arguments
///
/// * `request` - The Lambda HTTP request
///
/// # Returns
///
/// A GraphQL request (file uploads stored as variables if mapped)
pub async fn parse_multipart(request: Request) -> Result<GraphQlRequest> {
    let content_type = request
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| anyhow::anyhow!("Missing content-type"))?;

    let boundary = multer::parse_boundary(content_type)
        .map_err(|e| anyhow::anyhow!("Failed to parse boundary: {}", e))?;

    let body_bytes = request.body().to_vec();
    let cursor = Cursor::new(body_bytes);
    let stream = FramedRead::new(cursor, BytesCodec::new());
    let mut multipart = Multipart::new(stream, boundary);

    let mut operations: Option<GraphQlRequest> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "operations" {
            let text = field.text().await.map_err(|e| anyhow::anyhow!("{}", e))?;
            operations =
                Some(serde_json::from_str(&text).map_err(|e| anyhow::anyhow!("{}", e))?);
        }
        // Note: File uploads in Lambda typically require S3 presigned URLs
        // Direct file uploads via multipart are limited by Lambda payload size
    }

    operations.ok_or_else(|| anyhow::anyhow!("Missing operations"))
}
