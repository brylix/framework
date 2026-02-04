# File Uploads (S3)

Brylix provides an S3 provider for handling file uploads and downloads using presigned URLs. This is the recommended approach for serverless (Lambda) environments as it offloads file transfer directly to S3.

## Overview

Instead of uploading files through your Lambda function, the S3 provider generates presigned URLs that allow clients to upload/download files directly to/from S3. This approach:

- **Reduces Lambda costs** - No data transfer through Lambda
- **Improves performance** - Direct S3 transfers are faster
- **Scales automatically** - S3 handles concurrent uploads
- **Supports large files** - No Lambda payload limits

## Installation

Enable the `s3` feature in your `Cargo.toml`:

```toml
[dependencies]
brylix = { version = "0.2", features = ["s3"] }

# Or with multi-tenant support
brylix = { version = "0.2", features = ["s3", "multi-tenant"] }
```

## Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `S3_BUCKET` | Yes | - | S3 bucket name |
| `S3_REGION` | No | `us-east-1` | AWS region |
| `S3_UPLOAD_EXPIRES_SECS` | No | `3600` | Upload URL expiration (seconds) |
| `S3_DOWNLOAD_EXPIRES_SECS` | No | `3600` | Download URL expiration (seconds) |

AWS credentials are loaded via the standard credential chain:
1. Environment variables (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`)
2. IAM role (recommended for Lambda)
3. AWS profile (`~/.aws/credentials`)

### Example .env

```env
S3_BUCKET=my-app-uploads
S3_REGION=us-east-1
S3_UPLOAD_EXPIRES_SECS=3600
S3_DOWNLOAD_EXPIRES_SECS=7200
```

## Basic Usage

### Creating the Provider

```rust
use brylix::prelude::*;

// Create from environment (recommended)
let s3 = AwsS3Provider::try_from_env().await;

// Check if configured
if s3.is_configured() {
    println!("S3 bucket: {}", s3.bucket());
}
```

### Generating Upload URLs

```rust
use brylix::prelude::*;

let s3 = AwsS3Provider::try_from_env().await;

// Basic upload request
let request = PresignedUrlRequest::upload("products", "image.jpg");
let response = s3.generate_upload_url(request, None).await?;

// With content type and custom expiration
let request = PresignedUrlRequest::upload("avatars", "user123.png")
    .with_content_type("image/png")
    .with_expires_in(1800); // 30 minutes

let response = s3.generate_upload_url(request, None).await?;

println!("Upload URL: {}", response.url);
println!("S3 Key: {}", response.key);
println!("Expires at: {}", response.expires_at);
```

### Generating Download URLs

```rust
let response = s3.generate_download_url(
    "products",      // folder
    "image.jpg",     // filename
    None,            // tenant (None for single-tenant)
    Some(3600),      // custom expiration (optional)
).await?;

println!("Download URL: {}", response.url);
```

### Deleting Objects

```rust
s3.delete_object("products", "image.jpg", None).await?;
```

## Multi-Tenant File Organization

The S3 provider automatically handles tenant-based folder organization. When a tenant is provided, files are stored under a tenant-prefixed path.

### Path Structure

| Mode | Path Structure | Example |
|------|---------------|---------|
| Single-tenant | `/{folder}/{filename}` | `/products/image.jpg` |
| Multi-tenant | `/{tenant}/{folder}/{filename}` | `/acme/products/image.jpg` |

### Multi-Tenant Usage

```rust
use brylix::prelude::*;

// Get tenant from GraphQL context
let tenant_info = ctx.data_opt::<TenantInfo>();
let tenant_name = tenant_info.map(|t| t.name.as_str());

// Upload with tenant prefix
let request = PresignedUrlRequest::upload("products", "image.jpg")
    .with_content_type("image/jpeg");

let response = s3.generate_upload_url(request, tenant_name).await?;
// If tenant_name = Some("acme"), key = "acme/products/image.jpg"

// Download with tenant prefix
let response = s3.generate_download_url(
    "products",
    "image.jpg",
    tenant_name,
    None,
).await?;

// Delete with tenant prefix
s3.delete_object("products", "image.jpg", tenant_name).await?;
```

### Resulting S3 Structure

```
my-bucket/
├── acme/                    # Tenant: acme
│   ├── products/
│   │   └── image1.jpg
│   └── avatars/
│       └── user123.png
├── globex/                  # Tenant: globex
│   ├── products/
│   │   └── image1.jpg
│   └── documents/
│       └── report.pdf
└── initech/                 # Tenant: initech
    └── products/
        └── image1.jpg
```

## GraphQL Integration

### Adding S3Provider to Schema

```rust
use brylix::prelude::*;
use async_graphql::{Schema, EmptySubscription};

#[tokio::main]
async fn main() {
    let s3_provider = AwsS3Provider::try_from_env().await;

    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(s3_provider)
        .finish();

    // ... rest of setup
}
```

### Upload URL Mutation

```rust
use brylix::prelude::*;

#[derive(SimpleObject)]
pub struct UploadUrlResponse {
    pub url: String,
    pub key: String,
    pub method: String,
    pub expires_at: i64,
}

pub struct Mutation;

#[Object]
impl Mutation {
    /// Generate a presigned URL for file upload
    async fn get_upload_url(
        &self,
        ctx: &Context<'_>,
        folder: String,
        filename: String,
        content_type: Option<String>,
    ) -> Result<UploadUrlResponse> {
        // Require authentication
        let _ = require_auth(ctx)?;

        // Get S3 provider
        let s3 = ctx.data_unchecked::<AwsS3Provider>();

        // Get tenant for multi-tenant mode
        let tenant = ctx.data_opt::<TenantInfo>().map(|t| t.name.as_str());

        // Build request
        let mut request = PresignedUrlRequest::upload(&folder, &filename);
        if let Some(ct) = content_type {
            request = request.with_content_type(ct);
        }

        // Generate URL
        let response = s3.generate_upload_url(request, tenant)
            .await
            .map_err(gql_from_domain)?;

        Ok(UploadUrlResponse {
            url: response.url,
            key: response.key,
            method: response.method,
            expires_at: response.expires_at,
        })
    }

    /// Generate a presigned URL for file download
    async fn get_download_url(
        &self,
        ctx: &Context<'_>,
        folder: String,
        filename: String,
    ) -> Result<UploadUrlResponse> {
        let _ = require_auth(ctx)?;
        let s3 = ctx.data_unchecked::<AwsS3Provider>();
        let tenant = ctx.data_opt::<TenantInfo>().map(|t| t.name.as_str());

        let response = s3.generate_download_url(&folder, &filename, tenant, None)
            .await
            .map_err(gql_from_domain)?;

        Ok(UploadUrlResponse {
            url: response.url,
            key: response.key,
            method: response.method,
            expires_at: response.expires_at,
        })
    }

    /// Delete a file from S3
    async fn delete_file(
        &self,
        ctx: &Context<'_>,
        folder: String,
        filename: String,
    ) -> Result<bool> {
        let _ = require_auth(ctx)?;
        let s3 = ctx.data_unchecked::<AwsS3Provider>();
        let tenant = ctx.data_opt::<TenantInfo>().map(|t| t.name.as_str());

        s3.delete_object(&folder, &filename, tenant)
            .await
            .map_err(gql_from_domain)?;

        Ok(true)
    }
}
```

### GraphQL Queries

```graphql
# Get upload URL
mutation {
  getUploadUrl(
    folder: "products"
    filename: "image.jpg"
    contentType: "image/jpeg"
  ) {
    url
    key
    method
    expiresAt
  }
}

# Get download URL
mutation {
  getDownloadUrl(
    folder: "products"
    filename: "image.jpg"
  ) {
    url
    key
  }
}

# Delete file
mutation {
  deleteFile(
    folder: "products"
    filename: "image.jpg"
  )
}
```

## Client-Side Upload

After obtaining the presigned URL, clients upload directly to S3:

### JavaScript/TypeScript

```typescript
async function uploadFile(file: File, folder: string) {
  // 1. Get presigned URL from your API
  const { url, key } = await graphqlClient.request(GET_UPLOAD_URL, {
    folder,
    filename: file.name,
    contentType: file.type,
  });

  // 2. Upload directly to S3
  await fetch(url, {
    method: 'PUT',
    body: file,
    headers: {
      'Content-Type': file.type,
    },
  });

  // 3. Return the S3 key to store in your database
  return key;
}
```

### cURL

```bash
# Get presigned URL
URL=$(curl -X POST https://api.example.com/graphql \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query":"mutation { getUploadUrl(folder: \"products\", filename: \"image.jpg\", contentType: \"image/jpeg\") { url } }"}' \
  | jq -r '.data.getUploadUrl.url')

# Upload to S3
curl -X PUT "$URL" \
  -H "Content-Type: image/jpeg" \
  --data-binary @image.jpg
```

## Testing

### NoOp Provider for Tests

Use `NoOpS3Provider` for unit tests:

```rust
use brylix::prelude::*;

#[tokio::test]
async fn test_upload_mutation() {
    let s3 = NoOpS3Provider::new("test-bucket");

    let request = PresignedUrlRequest::upload("products", "test.jpg");
    let response = s3.generate_upload_url(request, None).await.unwrap();

    assert!(response.url.contains("test-bucket"));
    assert_eq!(response.key, "products/test.jpg");
    assert_eq!(response.method, "PUT");
}

#[tokio::test]
async fn test_multi_tenant_key() {
    let s3 = NoOpS3Provider::new("test-bucket");

    let request = PresignedUrlRequest::upload("products", "test.jpg");
    let response = s3.generate_upload_url(request, Some("acme")).await.unwrap();

    assert_eq!(response.key, "acme/products/test.jpg");
}
```

## S3 Bucket Configuration

### CORS Configuration

Enable CORS on your S3 bucket for browser uploads:

```json
{
  "CORSRules": [
    {
      "AllowedHeaders": ["*"],
      "AllowedMethods": ["PUT", "GET"],
      "AllowedOrigins": ["https://your-app.com"],
      "ExposeHeaders": ["ETag"],
      "MaxAgeSeconds": 3600
    }
  ]
}
```

### IAM Policy for Lambda

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "s3:PutObject",
        "s3:GetObject",
        "s3:DeleteObject"
      ],
      "Resource": "arn:aws:s3:::my-bucket/*"
    }
  ]
}
```

## Best Practices

1. **Use content type validation** - Validate file types before generating upload URLs
2. **Set appropriate expiration** - Shorter for uploads (15-60 min), longer for downloads
3. **Validate filenames** - Sanitize filenames to prevent path traversal
4. **Use unique filenames** - Consider UUID-based names to avoid collisions
5. **Store keys in database** - Save the S3 key after successful upload
6. **Handle upload failures** - Implement retry logic on the client side
7. **Clean up orphaned files** - Implement lifecycle policies or cleanup jobs

## API Reference

### PresignedUrlRequest

```rust
pub struct PresignedUrlRequest {
    pub folder: String,
    pub filename: String,
    pub content_type: Option<String>,
    pub expires_in_secs: Option<u64>,
}

impl PresignedUrlRequest {
    pub fn upload(folder: impl Into<String>, filename: impl Into<String>) -> Self;
    pub fn with_content_type(self, content_type: impl Into<String>) -> Self;
    pub fn with_expires_in(self, secs: u64) -> Self;
}
```

### PresignedUrlResponse

```rust
pub struct PresignedUrlResponse {
    pub url: String,        // Presigned URL
    pub method: String,     // HTTP method (PUT/GET)
    pub key: String,        // Full S3 key
    pub expires_at: i64,    // Unix timestamp
}
```

### S3Provider Trait

```rust
#[async_trait]
pub trait S3Provider: Send + Sync {
    async fn generate_upload_url(
        &self,
        request: PresignedUrlRequest,
        tenant: Option<&str>,
    ) -> DomainResult<PresignedUrlResponse>;

    async fn generate_download_url(
        &self,
        folder: &str,
        filename: &str,
        tenant: Option<&str>,
        expires_in_secs: Option<u64>,
    ) -> DomainResult<PresignedUrlResponse>;

    async fn delete_object(
        &self,
        folder: &str,
        filename: &str,
        tenant: Option<&str>,
    ) -> DomainResult<()>;

    fn build_key(&self, folder: &str, filename: &str, tenant: Option<&str>) -> String;
    fn is_configured(&self) -> bool;
    fn bucket(&self) -> &str;
}
```
