# Brylix CLI

Command-line tool for the Brylix framework. Create projects, generate code, and deploy to AWS Lambda.

## Installation

```bash
cargo install brylix-cli
```

## Commands

### Create New Project

```bash
# Basic project
brylix new my-api

# With multi-tenant support
brylix new my-api --multi-tenant

# With PostgreSQL
brylix new my-api --database postgres
```

### Generate Code

```bash
# Generate entity (SeaORM model)
brylix generate entity User

# Generate service layer
brylix generate service User

# Generate repository layer
brylix generate repository User

# Generate GraphQL resolver
brylix generate resolver User

# Generate migration
brylix generate migration create_users_table
```

### Development

```bash
# Run locally with hot reload
brylix dev

# Run database migrations
brylix migrate

# Create new migration
brylix migrate:generate migration_name
```

### Deployment

```bash
# Build for AWS Lambda (ARM64)
brylix build

# Deploy to AWS Lambda
brylix deploy
```

## Generated Project Structure

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
    └── src/
        └── lib.rs
```

## Requirements

- Rust 1.75+
- [cargo-lambda](https://www.cargo-lambda.info/) for local development and deployment

## Documentation

Full documentation is available at [docs.rs/brylix-cli](https://docs.rs/brylix-cli).

## License

Licensed under either of Apache License 2.0 or MIT License at your option.
