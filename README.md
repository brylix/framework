# Brylix Framework

A Rust framework for building GraphQL APIs on AWS Lambda with SeaORM and multi-tenant support.

## Features

- **GraphQL API** - Built on async-graphql with playground support
- **AWS Lambda** - Optimized for serverless deployment with cargo-lambda
- **SeaORM** - Type-safe database operations with MySQL/PostgreSQL support
- **Multi-tenant** - Pool-per-droplet architecture for SaaS applications
- **JWT Authentication** - Secure token-based authentication
- **Validation** - Built-in input validation utilities
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

## CLI Commands

```bash
# Create new project
brylix new my-api
brylix new my-api --multi-tenant
brylix new my-api --database postgres

# Generate code
brylix generate entity User
brylix generate service User
brylix generate repository User
brylix generate resolver User

# Development
brylix dev                    # Run with cargo-lambda watch
brylix migrate                # Run migrations
brylix migrate:generate name  # Create new migration

# Deployment
brylix build                  # Build for Lambda (ARM64)
brylix deploy                 # Deploy to AWS Lambda
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
brylix = "0.1"

# PostgreSQL
brylix = { version = "0.1", default-features = false, features = ["postgres", "playground"] }

# Multi-tenant
brylix = { version = "0.1", features = ["multi-tenant"] }

# Full features
brylix = { version = "0.1", features = ["full"] }
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
