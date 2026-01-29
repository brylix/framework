# Brylix Framework

**Brylix** is a Rust framework for building GraphQL APIs deployed on AWS Lambda. It provides a complete toolkit for building production-ready, multi-tenant SaaS applications.

## Features

- **GraphQL API** - Built on async-graphql with schema-first design
- **AWS Lambda** - Optimized for serverless deployment with cargo-lambda
- **SeaORM** - Type-safe database operations with MySQL and PostgreSQL
- **Multi-Tenant** - Pool-per-droplet architecture for efficient tenant isolation
- **JWT Auth** - Secure authentication with tenant-aware tokens
- **CLI Tool** - Code generation and project scaffolding

## Why Brylix?

### Type Safety

Rust's type system catches errors at compile time. No more runtime surprises.

```rust
// Errors auto-convert to proper GraphQL error codes
let user = UserService::get_by_id(&db, id)
    .await
    .map_err(gql_from_domain)?;
```

### Performance

Rust's zero-cost abstractions and Lambda's scale-to-zero model deliver both performance and cost efficiency.

### Developer Experience

The CLI generates boilerplate code, letting you focus on business logic:

```bash
brylix generate entity User
brylix generate service User
brylix generate resolver User
```

## Quick Example

```rust
use brylix::prelude::*;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn user(&self, ctx: &Context<'_>, id: i64) -> Result<UserDto> {
        let data = ctx.data_unchecked::<ContextData>();
        let user = UserService::get_by_id(&data.db, id)
            .await
            .map_err(gql_from_domain)?;
        Ok(UserDto::from(user))
    }
}
```

## Getting Started

Ready to build? Start with the [Installation Guide](./getting-started/installation.md).
