# brylix crate

Core framework library providing error handling, authentication, configuration, and GraphQL utilities.

## Installation

```toml
[dependencies]
brylix = { version = "0.2", features = ["mysql", "playground"] }
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `mysql` | MySQL database support (default) |
| `postgres` | PostgreSQL database support |
| `playground` | GraphQL Playground UI (default) |
| `multi-tenant` | Multi-tenant mode |
| `email` | SMTP email provider |
| `s3` | S3 presigned URL provider |
| `admin-override` | Temporary admin elevation |
| `full` | All features enabled |

## Modules

### `brylix::prelude`

Common imports for most use cases:

```rust
use brylix::prelude::*;

// Includes:
// - DomainError, DomainResult
// - gql_from_domain, gql_error, gql_bad_request, gql_not_found, gql_unauthorized
// - ContextData, require_auth
// - BrylixConfig
// - EmailMessage, EmailProvider, SmtpProvider (with email feature)
// - AwsS3Provider, S3Provider, PresignedUrlRequest, PresignedUrlResponse (with s3 feature)
```

### `brylix::config`

Configuration management:

```rust
use brylix::config::{BrylixConfig, BrylixConfigBuilder};

// From environment
let config = BrylixConfig::from_env()?;

// With builder
let config = BrylixConfigBuilder::new()
    .database_url("mysql://user:pass@localhost/db")
    .jwt_secret("secret")
    .build()?;

// Access values
let db_url = &config.database_url;
let secret = &config.jwt_secret;
```

### `brylix::errors`

Error types and helpers:

```rust
use brylix::errors::{DomainError, DomainResult};

// Domain errors
DomainError::InvalidCredentials
DomainError::Unauthorized
DomainError::Forbidden(String)
DomainError::NotFound(String)
DomainError::InvalidInput(String)
DomainError::DuplicateEntry(String)
DomainError::DatabaseError(String)
DomainError::Internal(String)

// GraphQL helpers
gql_from_domain(err)     // Convert DomainError
gql_error(code, msg)     // Custom error
gql_bad_request(msg)     // BAD_REQUEST
gql_not_found(msg)       // NOT_FOUND
gql_unauthorized()       // UNAUTHORIZED
```

### `brylix::auth`

Authentication utilities:

```rust
use brylix::auth;

// Password hashing
let hash = auth::hash_password("password")?;
let valid = auth::verify_password("password", &hash)?;

// JWT tokens
let token = auth::generate_token(&config, user_id, tenant)?;
let claims = auth::decode_token(&config, &token)?;
```

### `brylix::validation`

Input validation:

```rust
use brylix::validation;

validation::validate_email(&email)?;
validation::validate_password(&password)?;
validation::validate_not_empty(&field, "Field name")?;
```

### `brylix::db`

Database initialization:

```rust
use brylix::db;

let db = db::init_database(&config).await?;
```

### `brylix::graphql`

GraphQL context and guards:

```rust
use brylix::graphql::{ContextData, require_auth};

// In resolver
let data = ctx.data_unchecked::<ContextData>();
let db = &data.db;
let user_id = data.user_id;
let tenant = &data.tenant;

// Require authentication
let user_id = require_auth(ctx)?;
```

### `brylix::handler`

Lambda HTTP handler:

```rust
use brylix::handler;

// Single-tenant
let handler = handler::create_handler(schema, &config);

// Multi-tenant
let handler = handler::create_multi_tenant_handler(
    schema,
    master_db,
    tenant_manager,
    &config,
);
```

### `brylix::tenant`

Multi-tenant support:

```rust
use brylix::tenant::TenantManager;

let manager = TenantManager::new(config)?;
let db = manager.get_connection("tenant_name").await?;
```

### `brylix::provider::s3`

S3 presigned URL provider (requires `s3` feature):

```rust
use brylix::prelude::*;

// Create provider from environment
let s3 = AwsS3Provider::try_from_env().await;

// Generate upload URL
let request = PresignedUrlRequest::upload("products", "image.jpg")
    .with_content_type("image/jpeg")
    .with_expires_in(3600);
let response = s3.generate_upload_url(request, None).await?;

// Generate download URL
let response = s3.generate_download_url("products", "image.jpg", None, None).await?;

// Delete object
s3.delete_object("products", "image.jpg", None).await?;

// Multi-tenant: pass tenant name as second argument
let response = s3.generate_upload_url(request, Some("tenant1")).await?;
// response.key = "tenant1/products/image.jpg"
```

### `brylix::auth::admin_override`

Admin override for temporary elevated access (requires `admin-override` feature):

```rust
use brylix::prelude::*;

// Configuration
let config = AdminOverrideConfig::new("admin-secret".to_string())
    .with_expiry_secs(60);

// Issue a short-lived override token
let token = issue_admin_override_token(&config, admin_id, "Admin Name", Some("delete_invoice"))?;

// Validate an override token
let admin_override = validate_admin_override_token(&token, &config)?;
// admin_override.admin_id, admin_override.admin_name, admin_override.action

// In resolvers - get override from context
let override_info = get_admin_override(ctx); // Option<&AdminOverride>

// Require both user auth + admin override
let (user_id, admin_override) = require_auth_with_admin_override(ctx)?;

// Audit trail
let audit = AdminOverrideAudit {
    actor_user_id: 5,
    authorizer_admin_id: admin_override.admin_id,
    authorizer_name: admin_override.admin_name.clone(),
    action: admin_override.action.clone(),
};
audit.log();

// Header constant
assert_eq!(ADMIN_OVERRIDE_HEADER, "X-Admin-Override");
```

## Types

### BrylixConfig

```rust
pub struct BrylixConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub multi_tenant: bool,
    pub tenant_db_password: Option<String>,
    pub required_db_version: Option<i32>,
}
```

### ContextData

```rust
pub struct ContextData {
    pub db: DatabaseConnection,
    pub user_id: Option<i64>,
    pub tenant: Option<String>,
}
```

### Claims

```rust
pub struct Claims {
    pub sub: i64,           // User ID
    pub tenant: Option<String>,
    pub exp: i64,           // Expiration
    pub iat: i64,           // Issued at
}
```

### AdminOverrideConfig (admin-override feature)

```rust
pub struct AdminOverrideConfig {
    pub secret: String,         // JWT secret for override tokens
    pub expiry_secs: i64,       // Token lifetime (default: 60)
}
```

### AdminOverride (admin-override feature)

```rust
pub struct AdminOverride {
    pub admin_id: i64,          // Admin user ID
    pub admin_sub: String,      // Admin subject string
    pub admin_name: String,     // Admin display name
    pub action: Option<String>, // Action being authorized
}
```

### AdminOverrideAudit (admin-override feature)

```rust
pub struct AdminOverrideAudit {
    pub actor_user_id: i64,         // Who performed the action
    pub authorizer_admin_id: i64,   // Who authorized it
    pub authorizer_name: String,    // Authorizer's name
    pub action: Option<String>,     // What was authorized
}
```

### PresignedUrlRequest (s3 feature)

```rust
pub struct PresignedUrlRequest {
    pub folder: String,
    pub filename: String,
    pub content_type: Option<String>,
    pub expires_in_secs: Option<u64>,
}
```

### PresignedUrlResponse (s3 feature)

```rust
pub struct PresignedUrlResponse {
    pub url: String,        // Presigned URL
    pub method: String,     // HTTP method (PUT/GET)
    pub key: String,        // Full S3 key
    pub expires_at: i64,    // Unix timestamp
}
```
