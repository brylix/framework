# brylix crate

Core framework library providing error handling, authentication, configuration, and GraphQL utilities.

## Installation

```toml
[dependencies]
brylix = { version = "0.1", features = ["mysql", "playground"] }
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `mysql` | MySQL database support |
| `postgres` | PostgreSQL database support |
| `playground` | GraphQL Playground UI |
| `multi-tenant` | Multi-tenant mode |

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
