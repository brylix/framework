# Brylix

A Rust framework for building GraphQL APIs on AWS Lambda with SeaORM and multi-tenant support.

## Features

- **GraphQL API** - Built on async-graphql with playground support
- **AWS Lambda** - Optimized for serverless deployment with cargo-lambda
- **SeaORM** - Type-safe database operations with MySQL/PostgreSQL support
- **Multi-tenant** - Pool-per-droplet architecture for SaaS applications
- **JWT Authentication** - Secure token-based authentication with multi-role support
- **Validation** - Built-in input validation utilities
- **Email Provider** - SMTP email support with attachments
- **S3 Provider** - Presigned URL generation for file uploads/downloads
- **Pagination** - Generic pagination utilities for GraphQL connections
- **Helpers** - JSON parsing, timestamps, soft delete, and ID parsing utilities

## Installation

```toml
[dependencies]
brylix = "0.2"
```

## Quick Start

```rust
use brylix::prelude::*;
use async_graphql::{EmptySubscription, Schema};

#[tokio::main]
async fn main() -> Result<(), brylix::Error> {
    Brylix::builder()
        .config_from_env()
        .with_jwt_auth()
        .with_migrations::<migration::Migrator>()
        .build_schema(|| {
            Schema::build(
                graphql::Query,
                graphql::Mutation,
                EmptySubscription,
            )
            .finish()
        })
        .build()?
        .run()
        .await
}
```

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `mysql` | MySQL/MariaDB support via sqlx | Yes |
| `postgres` | PostgreSQL support via sqlx | No |
| `playground` | GraphQL Playground IDE | Yes |
| `multi-tenant` | Multi-tenant support | No |
| `email` | SMTP email with attachments | No |
| `s3` | S3 presigned URLs for file uploads | No |
| `full` | All features enabled | No |

```toml
# PostgreSQL instead of MySQL
brylix = { version = "0.2", default-features = false, features = ["postgres", "playground"] }

# Multi-tenant support
brylix = { version = "0.2", features = ["multi-tenant"] }

# Email support
brylix = { version = "0.2", features = ["email"] }

# S3 presigned URLs
brylix = { version = "0.2", features = ["s3"] }
```

## Utilities

### Pagination

```rust
use brylix::prelude::*;

// In a resolver:
async fn list_users(ctx: &Context<'_>, page: u64, per_page: u64) -> Result<Connection<UserDto>> {
    let data = ctx.data_unchecked::<ContextData>();
    let (items, total) = UserService::paginated(&data.db, page, per_page).await?;
    Ok(Connection::new(items, total, page, per_page))
}

// Or use IntoConnection trait with (Vec<T>, u64) tuples:
let result: (Vec<UserDto>, u64) = (items, total);
let connection = result.into_connection(page, per_page);
```

### GraphQL ID Parsing

```rust
use brylix::prelude::*;

// Parse string IDs to i64
let user_id = parse_gql_id("123")?;          // Ok(123)
let id = parse_gql_id_field("42", "user_id")?; // Custom error field name

// Or use the macro
let id = gql_id!("123");
let id = gql_id!("123", "user_id");
```

### JSON Column Helpers

```rust
use brylix::prelude::*;
use serde_json::json;

// Parse JSON database columns into typed structs
let json_col: Option<serde_json::Value> = Some(json!({"key": "value"}));
let parsed: Option<MyStruct> = json_col.parse_as();
let with_default: Vec<String> = json_col.parse_or_default();
```

### Timestamp Helpers

```rust
use brylix::prelude::*;

let now = utc_now(); // chrono::DateTime<Utc>

// Implement Timestamped for your ActiveModel:
impl Timestamped for users::ActiveModel {
    fn set_created_at(&mut self) { self.created_at = Set(utc_now()); }
    fn set_updated_at(&mut self) { self.updated_at = Set(utc_now()); }
}

let mut model = users::ActiveModel { .. };
model.set_timestamps(); // Sets both created_at and updated_at
```

### Soft Delete

```rust
use brylix::prelude::*;

// Use status constants
let active = status::ACTIVE;   // "active"
let deleted = status::DELETED;  // "deleted"

// Implement SoftDeletable for your models
impl SoftDeletable for Post {
    fn mark_deleted(&mut self) { self.status = status::DELETED.to_string(); }
    fn is_deleted(&self) -> bool { self.status == status::DELETED }
}
```

### Multi-Role Authentication

```rust
use brylix::prelude::*;

// Configure multiple JWT secrets
let jwt_config = MultiRoleJwtConfig::new()
    .add_role("user", std::env::var("JWT_SECRET").unwrap())
    .add_role("admin", std::env::var("ADMIN_JWT_SECRET").unwrap());

// Validate token to determine role
let role = jwt_config.validate(token); // Option<AuthRole>

// Set role in context
let ctx = ContextData::single_tenant(db, user_id, Some(AuthRole::Admin(1)));

// Use guards in resolvers
let admin_id = require_admin(ctx)?;       // Errors if not admin
let role = get_auth_role(ctx);            // Option<&AuthRole>
```

## Environment Variables

```env
# Required
DATABASE_URL=mysql://user:password@host/database
JWT_SECRET=your-secret-key
JWT_EXP_DAYS=7

# Email (optional, requires `email` feature)
SMTP_HOST=smtp.example.com
SMTP_PORT=465
SMTP_USER=your-email@example.com
SMTP_PASSWORD=your-password
SMTP_FROM_NAME=Your App Name
SMTP_FROM_EMAIL=noreply@example.com

# S3 (optional, requires `s3` feature)
S3_BUCKET=my-bucket-name
S3_REGION=us-east-1
S3_UPLOAD_EXPIRES_SECS=3600
S3_DOWNLOAD_EXPIRES_SECS=3600
# Custom credentials for local development (optional)
# If not set, falls back to default AWS credential chain (IAM role for Lambda)
S3_ACCESS_KEY_ID=your-access-key
S3_SECRET_ACCESS_KEY=your-secret-key
```

## Documentation

Full documentation is available at [docs.rs/brylix](https://docs.rs/brylix).

## License

Licensed under either of Apache License 2.0 or MIT License at your option.
