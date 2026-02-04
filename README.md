# Brylix Framework

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
- **CLI Tool** - Project scaffolding and code generation

## Quick Start

### Installation

```bash
# Install the CLI
cargo install brylix-cli

# Create a new project
brylix new my-api

# Or with multi-tenant support
brylix new my-api --multi-tenant
```

### Basic Usage

```rust
use brylix::prelude::*;
use async_graphql::{EmptySubscription, Schema};

mod graphql;
mod model;
mod repository;
mod service;

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

### Multi-Tenant Usage

```rust
use brylix::prelude::*;

#[tokio::main]
async fn main() -> Result<(), brylix::Error> {
    Brylix::builder()
        .config_from_env()
        .with_jwt_auth()
        .with_multi_tenant()  // Enable multi-tenant mode
        .with_migrations::<migration::Migrator>()
        .build_schema(|| /* ... */)
        .build()?
        .run()
        .await
}
```

### S3 Presigned URLs

Generate presigned URLs for file uploads and downloads. Supports multi-tenant file organization.

```rust
use brylix::prelude::*;

// Create S3 provider from environment
let s3 = AwsS3Provider::try_from_env().await;

// Generate upload URL (single-tenant)
let request = PresignedUrlRequest::upload("products", "image.jpg")
    .with_content_type("image/jpeg")
    .with_expires_in(3600);
let response = s3.generate_upload_url(request, None).await?;
// response.key = "products/image.jpg"

// Generate upload URL (multi-tenant)
let response = s3.generate_upload_url(request, Some("tenant1")).await?;
// response.key = "tenant1/products/image.jpg"

// Generate download URL
let response = s3.generate_download_url("products", "image.jpg", None, None).await?;

// Delete object
s3.delete_object("products", "image.jpg", None).await?;
```

**Multi-Tenant File Organization:**

| Mode | Path Structure | Example |
|------|---------------|---------|
| Single-tenant | `/{folder}/{filename}` | `/products/image.jpg` |
| Multi-tenant | `/{tenant}/{folder}/{filename}` | `/tenant1/products/image.jpg` |

## Environment Variables

### Required

```env
DATABASE_URL=mysql://user:password@host/database
JWT_SECRET=your-secret-key
JWT_EXP_DAYS=7
```

### Multi-Tenant (Optional)

```env
MULTI_TENANT_MODE=true
REQUIRED_DB_VERSION=1
TENANT_DB_PASSWORD=password-for-tenant-dbs
```

### Email/SMTP (Optional)

```env
SMTP_HOST=smtp.example.com
SMTP_PORT=465
SMTP_USER=your-email@example.com
SMTP_PASSWORD=your-password
SMTP_FROM_NAME=Your App Name
SMTP_FROM_EMAIL=noreply@example.com
```

### S3 Storage (Optional)

```env
S3_BUCKET=my-bucket-name
S3_REGION=us-east-1
S3_UPLOAD_EXPIRES_SECS=3600
S3_DOWNLOAD_EXPIRES_SECS=3600
```

AWS credentials are loaded via the standard credential chain (environment variables, IAM role, or AWS profile).

## CLI Commands

### Getting Help

```bash
# Show all available commands
brylix --help
brylix -h

# Show help for a specific command
brylix new --help
brylix generate --help
brylix dev --help

# Show help for subcommands
brylix generate entity --help
```

### Available Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `new` | - | Create a new Brylix project |
| `generate` | `g` | Generate code (entity, service, repository, resolver) |
| `dev` | - | Run development server with hot reload |
| `build` | - | Build for AWS Lambda deployment |
| `deploy` | - | Deploy to AWS Lambda |
| `migrate` | - | Run database migrations |
| `test` | - | Run tests |

### Examples

```bash
# Create new project
brylix new my-api
brylix new my-api --multi-tenant
brylix new my-api --database postgres

# Generate code
brylix generate entity User
brylix g service User          # Using alias
brylix generate repository User
brylix generate resolver User
brylix generate all User       # Generate all files for an entity

# Development
brylix dev                     # Run with cargo-lambda watch
brylix dev --port 8080         # Custom port

# Testing
brylix test                    # Run all tests
brylix test --watch            # Watch mode
brylix test --unit             # Unit tests only

# Migrations
brylix migrate                 # Run migrations
brylix migrate --generate name # Create new migration
brylix migrate --down          # Rollback last migration

# Deployment
brylix build                   # Build for Lambda (ARM64)
brylix build --arm64           # Explicitly build for ARM64
brylix deploy                  # Deploy to AWS Lambda
brylix deploy --profile prod   # Deploy with specific AWS profile
```

## Project Structure

```
my-api/
├── Cargo.toml
├── .env
├── src/
│   ├── main.rs
│   ├── graphql/
│   │   ├── mod.rs
│   │   ├── query.rs
│   │   ├── mutation.rs
│   │   └── types.rs
│   ├── service/
│   ├── repository/
│   └── model/
└── migration/
```

## Feature Flags

```toml
# MySQL (default)
brylix = "0.2"

# PostgreSQL
brylix = { version = "0.2", default-features = false, features = ["postgres", "playground"] }

# Multi-tenant
brylix = { version = "0.2", features = ["multi-tenant"] }

# Email support
brylix = { version = "0.2", features = ["email"] }

# S3 presigned URLs
brylix = { version = "0.2", features = ["s3"] }

# Full features (includes all: mysql, postgres, playground, multi-tenant, email, s3)
brylix = { version = "0.2", features = ["full"] }
```

## Documentation

### Getting Started
- [Installation](docs/src/getting-started/installation.md)
- [Quick Start](docs/src/getting-started/quick-start.md)
- [Project Structure](docs/src/getting-started/project-structure.md)

### Guides
- [Configuration](docs/src/guides/configuration.md)
- [GraphQL](docs/src/guides/graphql.md)
- [Authentication](docs/src/guides/authentication.md)
- [Database](docs/src/guides/database.md)
- [Multi-Tenancy](docs/src/guides/multi-tenancy.md)
- [File Uploads (S3)](docs/src/guides/file-uploads.md)
- [Error Handling](docs/src/guides/error-handling.md)
- [Deployment](docs/src/guides/deployment.md)

### API Reference
- [Brylix Crate](docs/src/api/brylix.md)
- [CLI Reference](docs/src/api/cli.md)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
