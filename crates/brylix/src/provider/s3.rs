//! S3 provider for generating presigned URLs.
//!
//! This module provides an `S3Provider` trait and an AWS S3 implementation
//! for generating presigned URLs for file uploads and downloads.
//!
//! # Feature
//!
//! This module requires the `s3` feature to be enabled.
//!
//! # Multi-Tenant Support
//!
//! The provider automatically handles tenant-based folder organization:
//!
//! | Mode | Path Structure | Example |
//! |------|---------------|---------|
//! | Single-tenant | `/{folder}/{filename}` | `/products/image.jpg` |
//! | Multi-tenant | `/{tenant}/{folder}/{filename}` | `/tenant1/products/image.jpg` |
//!
//! # Usage
//!
//! ```rust,ignore
//! use brylix::provider::s3::{AwsS3Provider, PresignedUrlRequest, S3Provider};
//!
//! // Create provider from environment variables
//! let provider = AwsS3Provider::try_from_env().await;
//!
//! // Generate upload URL (single-tenant)
//! let request = PresignedUrlRequest::upload("products", "image.jpg")
//!     .with_content_type("image/jpeg");
//! let response = provider.generate_upload_url(request, None).await?;
//!
//! // Generate upload URL (multi-tenant)
//! let response = provider.generate_upload_url(request, Some("tenant1")).await?;
//! // response.key = "tenant1/products/image.jpg"
//! ```
//!
//! # Environment Variables
//!
//! | Variable | Required | Default | Description |
//! |----------|----------|---------|-------------|
//! | `S3_BUCKET` | Yes | - | S3 bucket name |
//! | `S3_REGION` | No | us-east-1 | AWS region |
//! | `S3_UPLOAD_EXPIRES_SECS` | No | 3600 | Upload URL expiration |
//! | `S3_DOWNLOAD_EXPIRES_SECS` | No | 3600 | Download URL expiration |
//!
//! AWS credentials are loaded via the standard credential chain (env vars, IAM role, profile).

use async_trait::async_trait;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client;
use std::time::Duration;

use crate::errors::{DomainError, DomainResult};

/// Default expiration time for upload URLs (1 hour)
const DEFAULT_UPLOAD_EXPIRES_SECS: u64 = 3600;

/// Default expiration time for download URLs (1 hour)
const DEFAULT_DOWNLOAD_EXPIRES_SECS: u64 = 3600;

/// Request for generating a presigned URL.
///
/// Use the builder-style methods to construct a request.
///
/// # Example
///
/// ```rust
/// use brylix::provider::s3::PresignedUrlRequest;
///
/// let request = PresignedUrlRequest::upload("products", "image.jpg")
///     .with_content_type("image/jpeg")
///     .with_max_size(5 * 1024 * 1024) // 5 MB
///     .with_expires_in(7200); // 2 hours
/// ```
#[derive(Debug, Clone)]
pub struct PresignedUrlRequest {
    /// Subfolder path (e.g., "products", "avatars", "documents/reports")
    pub folder: String,
    /// Filename (e.g., "image.jpg")
    pub filename: String,
    /// Content type (e.g., "image/jpeg")
    pub content_type: Option<String>,
    /// URL expiration in seconds
    pub expires_in_secs: Option<u64>,
    /// Maximum allowed file size in bytes (for client-side validation)
    pub max_size_bytes: Option<u64>,
}

impl PresignedUrlRequest {
    /// Create an upload request with folder and filename.
    ///
    /// # Arguments
    ///
    /// * `folder` - Subfolder path (e.g., "products", "avatars")
    /// * `filename` - Filename (e.g., "image.jpg")
    ///
    /// # Example
    ///
    /// ```rust
    /// use brylix::provider::s3::PresignedUrlRequest;
    ///
    /// let request = PresignedUrlRequest::upload("products", "abc123.jpg");
    /// ```
    #[must_use]
    pub fn upload(folder: impl Into<String>, filename: impl Into<String>) -> Self {
        Self {
            folder: folder.into(),
            filename: filename.into(),
            content_type: None,
            expires_in_secs: None,
            max_size_bytes: None,
        }
    }

    /// Set the content type for the upload.
    ///
    /// # Arguments
    ///
    /// * `content_type` - MIME type (e.g., "image/jpeg", "application/pdf")
    #[must_use]
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    /// Set the URL expiration time in seconds.
    ///
    /// # Arguments
    ///
    /// * `secs` - Expiration time in seconds
    #[must_use]
    pub fn with_expires_in(mut self, secs: u64) -> Self {
        self.expires_in_secs = Some(secs);
        self
    }

    /// Set the maximum allowed file size in bytes.
    ///
    /// This value is returned in the response for client-side validation.
    /// The server should also validate the file size after upload confirmation.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Maximum file size in bytes
    ///
    /// # Example
    ///
    /// ```rust
    /// use brylix::provider::s3::PresignedUrlRequest;
    ///
    /// let request = PresignedUrlRequest::upload("avatars", "photo.jpg")
    ///     .with_max_size(5 * 1024 * 1024); // 5 MB limit
    /// ```
    #[must_use]
    pub fn with_max_size(mut self, bytes: u64) -> Self {
        self.max_size_bytes = Some(bytes);
        self
    }
}

/// Response containing presigned URL details.
///
/// Contains all information needed to perform the upload/download operation.
#[derive(Debug, Clone)]
pub struct PresignedUrlResponse {
    /// The presigned URL for upload/download
    pub url: String,
    /// HTTP method (PUT for upload, GET for download)
    pub method: String,
    /// Full S3 key (includes tenant prefix if multi-tenant)
    pub key: String,
    /// URL expiration timestamp (Unix seconds)
    pub expires_at: i64,
    /// Maximum allowed file size in bytes (for client-side validation)
    pub max_size_bytes: Option<u64>,
}

/// Trait for S3 operations with presigned URLs.
///
/// Implement this trait to create custom S3 providers or mock implementations.
#[async_trait]
pub trait S3Provider: Send + Sync {
    /// Generate a presigned URL for uploading a file.
    ///
    /// # Arguments
    ///
    /// * `request` - The upload request details
    /// * `tenant` - Optional tenant name for multi-tenant mode
    ///
    /// # Path Structure
    ///
    /// - Single-tenant: `/{folder}/{filename}`
    /// - Multi-tenant: `/{tenant}/{folder}/{filename}`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Single-tenant: /products/abc123.jpg
    /// let request = PresignedUrlRequest::upload("products", "abc123.jpg");
    /// let response = provider.generate_upload_url(request, None).await?;
    ///
    /// // Multi-tenant: /tenant1/products/abc123.jpg
    /// let response = provider.generate_upload_url(request, Some("tenant1")).await?;
    /// ```
    async fn generate_upload_url(
        &self,
        request: PresignedUrlRequest,
        tenant: Option<&str>,
    ) -> DomainResult<PresignedUrlResponse>;

    /// Generate a presigned URL for downloading a file.
    ///
    /// # Arguments
    ///
    /// * `folder` - Subfolder path
    /// * `filename` - Filename
    /// * `tenant` - Optional tenant name for multi-tenant mode
    /// * `expires_in_secs` - Optional custom expiration time
    async fn generate_download_url(
        &self,
        folder: &str,
        filename: &str,
        tenant: Option<&str>,
        expires_in_secs: Option<u64>,
    ) -> DomainResult<PresignedUrlResponse>;

    /// Delete an object from S3.
    ///
    /// # Arguments
    ///
    /// * `folder` - Subfolder path
    /// * `filename` - Filename
    /// * `tenant` - Optional tenant name for multi-tenant mode
    async fn delete_object(
        &self,
        folder: &str,
        filename: &str,
        tenant: Option<&str>,
    ) -> DomainResult<()>;

    /// Build the full S3 key with optional tenant prefix.
    ///
    /// # Arguments
    ///
    /// * `folder` - Subfolder path
    /// * `filename` - Filename
    /// * `tenant` - Optional tenant name
    ///
    /// # Returns
    ///
    /// - With tenant: `{tenant}/{folder}/{filename}`
    /// - Without tenant: `{folder}/{filename}`
    fn build_key(&self, folder: &str, filename: &str, tenant: Option<&str>) -> String {
        let clean_folder = folder.trim_matches('/');
        match tenant {
            Some(t) => format!("{}/{}/{}", t, clean_folder, filename),
            None => format!("{}/{}", clean_folder, filename),
        }
    }

    /// Check if the provider is configured and ready to use.
    fn is_configured(&self) -> bool;

    /// Get the bucket name.
    fn bucket(&self) -> &str;
}

/// S3 configuration loaded from environment variables.
#[derive(Debug, Clone)]
struct S3Config {
    bucket: String,
    region: String,
    upload_expires_secs: u64,
    download_expires_secs: u64,
}

impl S3Config {
    fn from_env() -> DomainResult<Self> {
        let bucket = std::env::var("S3_BUCKET")
            .map_err(|_| DomainError::Internal("S3_BUCKET not set".to_string()))?;

        let region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());

        let upload_expires_secs = std::env::var("S3_UPLOAD_EXPIRES_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_UPLOAD_EXPIRES_SECS);

        let download_expires_secs = std::env::var("S3_DOWNLOAD_EXPIRES_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_DOWNLOAD_EXPIRES_SECS);

        Ok(Self {
            bucket,
            region,
            upload_expires_secs,
            download_expires_secs,
        })
    }
}

/// AWS S3 provider implementation.
///
/// Uses the AWS SDK to generate presigned URLs for S3 operations.
///
/// # Example
///
/// ```rust,ignore
/// use brylix::provider::s3::{AwsS3Provider, PresignedUrlRequest, S3Provider};
///
/// let provider = AwsS3Provider::try_from_env().await;
///
/// if provider.is_configured() {
///     let request = PresignedUrlRequest::upload("products", "image.jpg")
///         .with_content_type("image/jpeg");
///     let response = provider.generate_upload_url(request, None).await?;
///     println!("Upload URL: {}", response.url);
/// }
/// ```
pub struct AwsS3Provider {
    client: Client,
    config: S3Config,
    configured: bool,
}

impl AwsS3Provider {
    /// Create a new AWS S3 provider with the given configuration.
    ///
    /// Supports two credential modes:
    /// 1. **Custom credentials**: If `S3_ACCESS_KEY_ID` and `S3_SECRET_ACCESS_KEY` are set,
    ///    uses those (useful for local development).
    /// 2. **Default chain**: Falls back to standard AWS credential chain (IAM role, env vars, profile).
    ///    This is the recommended approach for Lambda deployments.
    async fn new(config: S3Config) -> DomainResult<Self> {
        use aws_credential_types::Credentials;
        use aws_credential_types::provider::SharedCredentialsProvider;

        let region = aws_sdk_s3::config::Region::new(config.region.clone());

        // Check for custom S3 credentials (for local development)
        let aws_config = match (
            std::env::var("S3_ACCESS_KEY_ID"),
            std::env::var("S3_SECRET_ACCESS_KEY"),
        ) {
            (Ok(access_key), Ok(secret_key)) => {
                tracing::debug!("Using custom S3 credentials from S3_ACCESS_KEY_ID/S3_SECRET_ACCESS_KEY");
                let credentials = Credentials::new(
                    access_key,
                    secret_key,
                    None, // session token
                    None, // expiry
                    "s3_env_credentials",
                );
                aws_config::defaults(aws_config::BehaviorVersion::latest())
                    .region(region)
                    .credentials_provider(SharedCredentialsProvider::new(credentials))
                    .load()
                    .await
            }
            _ => {
                // Fall back to default AWS credential chain (IAM role for Lambda)
                tracing::debug!("Using default AWS credential chain");
                aws_config::defaults(aws_config::BehaviorVersion::latest())
                    .region(region)
                    .load()
                    .await
            }
        };

        let client = Client::new(&aws_config);

        Ok(Self {
            client,
            config,
            configured: true,
        })
    }

    /// Create a new AWS S3 provider from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are not set.
    async fn from_env() -> DomainResult<Self> {
        let config = S3Config::from_env()?;
        Self::new(config).await
    }

    /// Try to create an AWS S3 provider from environment variables.
    ///
    /// Returns an unconfigured provider if environment variables are not set,
    /// allowing graceful degradation. The unconfigured provider will return
    /// errors when operations are attempted.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let provider = AwsS3Provider::try_from_env().await;
    /// if provider.is_configured() {
    ///     // Use the provider
    /// }
    /// ```
    #[must_use]
    pub async fn try_from_env() -> Self {
        match Self::from_env().await {
            Ok(provider) => provider,
            Err(e) => {
                tracing::warn!("S3 provider not configured: {}", e);
                Self::unconfigured().await
            }
        }
    }

    /// Create an unconfigured provider that returns errors on operations.
    async fn unconfigured() -> Self {
        // Create a minimal config for the unconfigured state
        let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_sdk_s3::config::Region::new("us-east-1"))
            .load()
            .await;

        Self {
            client: Client::new(&aws_config),
            config: S3Config {
                bucket: String::new(),
                region: String::new(),
                upload_expires_secs: 0,
                download_expires_secs: 0,
            },
            configured: false,
        }
    }

    /// Get the current Unix timestamp plus the given duration.
    fn expires_at(duration_secs: u64) -> i64 {
        chrono::Utc::now().timestamp() + duration_secs as i64
    }
}

#[async_trait]
impl S3Provider for AwsS3Provider {
    async fn generate_upload_url(
        &self,
        request: PresignedUrlRequest,
        tenant: Option<&str>,
    ) -> DomainResult<PresignedUrlResponse> {
        if !self.configured {
            return Err(DomainError::ProviderNotConfigured);
        }

        let key = self.build_key(&request.folder, &request.filename, tenant);
        let expires_in = request
            .expires_in_secs
            .unwrap_or(self.config.upload_expires_secs);

        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(expires_in))
            .map_err(|e| DomainError::ExternalService(format!("Invalid presign config: {}", e)))?;

        let mut put_request = self
            .client
            .put_object()
            .bucket(&self.config.bucket)
            .key(&key);

        if let Some(content_type) = &request.content_type {
            put_request = put_request.content_type(content_type);
        }

        let presigned = put_request
            .presigned(presigning_config)
            .await
            .map_err(|e| DomainError::ExternalService(format!("Failed to generate presigned URL: {}", e)))?;

        Ok(PresignedUrlResponse {
            url: presigned.uri().to_string(),
            method: "PUT".to_string(),
            key,
            expires_at: Self::expires_at(expires_in),
            max_size_bytes: request.max_size_bytes,
        })
    }

    async fn generate_download_url(
        &self,
        folder: &str,
        filename: &str,
        tenant: Option<&str>,
        expires_in_secs: Option<u64>,
    ) -> DomainResult<PresignedUrlResponse> {
        if !self.configured {
            return Err(DomainError::ProviderNotConfigured);
        }

        let key = self.build_key(folder, filename, tenant);
        let expires_in = expires_in_secs.unwrap_or(self.config.download_expires_secs);

        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(expires_in))
            .map_err(|e| DomainError::ExternalService(format!("Invalid presign config: {}", e)))?;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.config.bucket)
            .key(&key)
            .presigned(presigning_config)
            .await
            .map_err(|e| DomainError::ExternalService(format!("Failed to generate presigned URL: {}", e)))?;

        Ok(PresignedUrlResponse {
            url: presigned.uri().to_string(),
            method: "GET".to_string(),
            key,
            expires_at: Self::expires_at(expires_in),
            max_size_bytes: None,
        })
    }

    async fn delete_object(
        &self,
        folder: &str,
        filename: &str,
        tenant: Option<&str>,
    ) -> DomainResult<()> {
        if !self.configured {
            return Err(DomainError::ProviderNotConfigured);
        }

        let key = self.build_key(folder, filename, tenant);

        self.client
            .delete_object()
            .bucket(&self.config.bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| DomainError::ExternalService(format!("Failed to delete object: {}", e)))?;

        tracing::info!("Deleted S3 object: {}/{}", self.config.bucket, key);
        Ok(())
    }

    fn is_configured(&self) -> bool {
        self.configured
    }

    fn bucket(&self) -> &str {
        &self.config.bucket
    }
}

/// A no-op S3 provider for testing or when S3 is not needed.
///
/// This provider logs operations instead of executing them.
///
/// # Example
///
/// ```rust
/// use brylix::provider::s3::{NoOpS3Provider, PresignedUrlRequest, S3Provider};
///
/// let provider = NoOpS3Provider::new("test-bucket");
/// // Operations will be logged but not executed
/// ```
pub struct NoOpS3Provider {
    bucket: String,
}

impl NoOpS3Provider {
    /// Create a new no-op S3 provider with a bucket name.
    #[must_use]
    pub fn new(bucket: impl Into<String>) -> Self {
        Self {
            bucket: bucket.into(),
        }
    }
}

impl Default for NoOpS3Provider {
    fn default() -> Self {
        Self::new("noop-bucket")
    }
}

#[async_trait]
impl S3Provider for NoOpS3Provider {
    async fn generate_upload_url(
        &self,
        request: PresignedUrlRequest,
        tenant: Option<&str>,
    ) -> DomainResult<PresignedUrlResponse> {
        let key = self.build_key(&request.folder, &request.filename, tenant);
        let expires_in = request.expires_in_secs.unwrap_or(DEFAULT_UPLOAD_EXPIRES_SECS);

        tracing::debug!(
            "NoOpS3Provider: Would generate upload URL for {}/{} (content_type: {:?})",
            self.bucket,
            key,
            request.content_type
        );

        Ok(PresignedUrlResponse {
            url: format!("https://{}.s3.amazonaws.com/{}?mock=true", self.bucket, key),
            method: "PUT".to_string(),
            key,
            expires_at: chrono::Utc::now().timestamp() + expires_in as i64,
            max_size_bytes: request.max_size_bytes,
        })
    }

    async fn generate_download_url(
        &self,
        folder: &str,
        filename: &str,
        tenant: Option<&str>,
        expires_in_secs: Option<u64>,
    ) -> DomainResult<PresignedUrlResponse> {
        let key = self.build_key(folder, filename, tenant);
        let expires_in = expires_in_secs.unwrap_or(DEFAULT_DOWNLOAD_EXPIRES_SECS);

        tracing::debug!(
            "NoOpS3Provider: Would generate download URL for {}/{}",
            self.bucket,
            key
        );

        Ok(PresignedUrlResponse {
            url: format!("https://{}.s3.amazonaws.com/{}?mock=true", self.bucket, key),
            method: "GET".to_string(),
            key,
            expires_at: chrono::Utc::now().timestamp() + expires_in as i64,
            max_size_bytes: None,
        })
    }

    async fn delete_object(
        &self,
        folder: &str,
        filename: &str,
        tenant: Option<&str>,
    ) -> DomainResult<()> {
        let key = self.build_key(folder, filename, tenant);

        tracing::debug!(
            "NoOpS3Provider: Would delete object {}/{}",
            self.bucket,
            key
        );

        Ok(())
    }

    fn is_configured(&self) -> bool {
        false
    }

    fn bucket(&self) -> &str {
        &self.bucket
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presigned_url_request_upload() {
        let request = PresignedUrlRequest::upload("products", "image.jpg");
        assert_eq!(request.folder, "products");
        assert_eq!(request.filename, "image.jpg");
        assert!(request.content_type.is_none());
        assert!(request.expires_in_secs.is_none());
    }

    #[test]
    fn test_presigned_url_request_with_content_type() {
        let request = PresignedUrlRequest::upload("products", "image.jpg")
            .with_content_type("image/jpeg");
        assert_eq!(request.content_type, Some("image/jpeg".to_string()));
    }

    #[test]
    fn test_presigned_url_request_with_expires_in() {
        let request = PresignedUrlRequest::upload("products", "image.jpg")
            .with_expires_in(7200);
        assert_eq!(request.expires_in_secs, Some(7200));
    }

    #[test]
    fn test_presigned_url_request_builder_chain() {
        let request = PresignedUrlRequest::upload("avatars", "user123.png")
            .with_content_type("image/png")
            .with_expires_in(1800);

        assert_eq!(request.folder, "avatars");
        assert_eq!(request.filename, "user123.png");
        assert_eq!(request.content_type, Some("image/png".to_string()));
        assert_eq!(request.expires_in_secs, Some(1800));
    }

    #[test]
    fn test_build_key_without_tenant() {
        let provider = NoOpS3Provider::new("test-bucket");
        let key = provider.build_key("products", "image.jpg", None);
        assert_eq!(key, "products/image.jpg");
    }

    #[test]
    fn test_build_key_with_tenant() {
        let provider = NoOpS3Provider::new("test-bucket");
        let key = provider.build_key("products", "image.jpg", Some("tenant1"));
        assert_eq!(key, "tenant1/products/image.jpg");
    }

    #[test]
    fn test_build_key_trims_slashes() {
        let provider = NoOpS3Provider::new("test-bucket");
        let key = provider.build_key("/products/", "image.jpg", None);
        assert_eq!(key, "products/image.jpg");
    }

    #[test]
    fn test_build_key_nested_folder() {
        let provider = NoOpS3Provider::new("test-bucket");
        let key = provider.build_key("documents/reports", "2024-q1.pdf", Some("acme"));
        assert_eq!(key, "acme/documents/reports/2024-q1.pdf");
    }

    #[tokio::test]
    async fn test_noop_provider_upload_url() {
        let provider = NoOpS3Provider::new("test-bucket");
        let request = PresignedUrlRequest::upload("products", "image.jpg")
            .with_content_type("image/jpeg");

        let response = provider.generate_upload_url(request, None).await.unwrap();

        assert!(response.url.contains("test-bucket"));
        assert!(response.url.contains("products/image.jpg"));
        assert_eq!(response.method, "PUT");
        assert_eq!(response.key, "products/image.jpg");
        assert!(response.expires_at > 0);
    }

    #[tokio::test]
    async fn test_noop_provider_upload_url_with_tenant() {
        let provider = NoOpS3Provider::new("test-bucket");
        let request = PresignedUrlRequest::upload("products", "image.jpg");

        let response = provider
            .generate_upload_url(request, Some("tenant1"))
            .await
            .unwrap();

        assert!(response.url.contains("tenant1/products/image.jpg"));
        assert_eq!(response.key, "tenant1/products/image.jpg");
    }

    #[tokio::test]
    async fn test_noop_provider_download_url() {
        let provider = NoOpS3Provider::new("test-bucket");

        let response = provider
            .generate_download_url("products", "image.jpg", None, None)
            .await
            .unwrap();

        assert!(response.url.contains("test-bucket"));
        assert_eq!(response.method, "GET");
        assert_eq!(response.key, "products/image.jpg");
    }

    #[tokio::test]
    async fn test_noop_provider_download_url_with_tenant() {
        let provider = NoOpS3Provider::new("test-bucket");

        let response = provider
            .generate_download_url("products", "image.jpg", Some("acme"), Some(600))
            .await
            .unwrap();

        assert_eq!(response.key, "acme/products/image.jpg");
    }

    #[tokio::test]
    async fn test_noop_provider_delete_object() {
        let provider = NoOpS3Provider::new("test-bucket");
        let result = provider
            .delete_object("products", "image.jpg", None)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_noop_provider_delete_object_with_tenant() {
        let provider = NoOpS3Provider::new("test-bucket");
        let result = provider
            .delete_object("products", "image.jpg", Some("tenant1"))
            .await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_noop_provider_not_configured() {
        let provider = NoOpS3Provider::new("test-bucket");
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_noop_provider_bucket() {
        let provider = NoOpS3Provider::new("my-bucket");
        assert_eq!(provider.bucket(), "my-bucket");
    }

    #[test]
    fn test_noop_provider_default() {
        let provider = NoOpS3Provider::default();
        assert_eq!(provider.bucket(), "noop-bucket");
    }

    #[test]
    fn test_s3_config_missing_bucket() {
        std::env::remove_var("S3_BUCKET");
        let result = S3Config::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_s3_config_with_defaults() {
        std::env::set_var("S3_BUCKET", "test-bucket");
        std::env::remove_var("S3_REGION");
        std::env::remove_var("S3_UPLOAD_EXPIRES_SECS");
        std::env::remove_var("S3_DOWNLOAD_EXPIRES_SECS");

        let config = S3Config::from_env().unwrap();
        assert_eq!(config.bucket, "test-bucket");
        assert_eq!(config.region, "us-east-1");
        assert_eq!(config.upload_expires_secs, DEFAULT_UPLOAD_EXPIRES_SECS);
        assert_eq!(config.download_expires_secs, DEFAULT_DOWNLOAD_EXPIRES_SECS);

        std::env::remove_var("S3_BUCKET");
    }

    #[test]
    fn test_s3_config_with_custom_values() {
        std::env::set_var("S3_BUCKET", "custom-bucket");
        std::env::set_var("S3_REGION", "eu-west-1");
        std::env::set_var("S3_UPLOAD_EXPIRES_SECS", "7200");
        std::env::set_var("S3_DOWNLOAD_EXPIRES_SECS", "1800");

        let config = S3Config::from_env().unwrap();
        assert_eq!(config.bucket, "custom-bucket");
        assert_eq!(config.region, "eu-west-1");
        assert_eq!(config.upload_expires_secs, 7200);
        assert_eq!(config.download_expires_secs, 1800);

        std::env::remove_var("S3_BUCKET");
        std::env::remove_var("S3_REGION");
        std::env::remove_var("S3_UPLOAD_EXPIRES_SECS");
        std::env::remove_var("S3_DOWNLOAD_EXPIRES_SECS");
    }

    #[tokio::test]
    async fn test_aws_s3_provider_unconfigured() {
        std::env::remove_var("S3_BUCKET");

        let provider = AwsS3Provider::try_from_env().await;
        assert!(!provider.is_configured());
    }

    #[tokio::test]
    async fn test_aws_s3_provider_unconfigured_returns_error() {
        std::env::remove_var("S3_BUCKET");

        let provider = AwsS3Provider::try_from_env().await;
        let request = PresignedUrlRequest::upload("products", "image.jpg");

        let result = provider.generate_upload_url(request, None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::ProviderNotConfigured));
    }
}
