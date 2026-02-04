# Brylix

A Rust framework for building GraphQL APIs on AWS Lambda with SeaORM and multi-tenant support.

## Features

- **GraphQL API** - Built on async-graphql with playground support
- **AWS Lambda** - Optimized for serverless deployment with cargo-lambda
- **SeaORM** - Type-safe database operations with MySQL/PostgreSQL support
- **Multi-tenant** - Pool-per-droplet architecture for SaaS applications
- **JWT Authentication** - Secure token-based authentication
- **Validation** - Built-in input validation utilities
- **Email Provider** - SMTP email support with attachments
- **S3 Provider** - Presigned URL generation for file uploads/downloads

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
