# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.3] - 2025-01-31

### Added
- New `email` feature for sending emails via SMTP
- `EmailProvider` trait for custom email provider implementations
- `SmtpProvider` - SMTP implementation with TLS support
- `EmailMessage` - Builder for constructing email messages
- `EmailAttachment` - Support for file attachments with convenience methods for common file types (PDF, CSV, Excel, JSON, images, ZIP)
- `NoOpEmailProvider` - Testing/disabled email provider
- New environment variables for SMTP configuration:
  - `SMTP_HOST` - SMTP server hostname (required)
  - `SMTP_PORT` - SMTP server port (default: 465)
  - `SMTP_USER` - SMTP username (required)
  - `SMTP_PASSWORD` - SMTP password (required)
  - `SMTP_FROM_NAME` - Sender display name (default: "Brylix")
  - `SMTP_FROM_EMAIL` - Sender email address (default: SMTP_USER)

### Changed
- Updated `full` feature to include `email`

## [0.2.2] - 2025-01-31

### Fixed
- Run tests sequentially to avoid DB migration conflicts

## [0.2.1] - 2025-01-31

### Added
- Added vendored OpenSSL to brylix crate - projects no longer need to add `openssl = { version = "0.10", features = ["vendored"] }` manually for Lambda ARM64 cross-compilation

## [0.2.0] - 2025-01-31

### Changed
- **BREAKING**: Upgraded `lambda_http` from 0.17 to 1.0.2
- **BREAKING**: Upgraded `lambda_runtime` from 0.14 to 1.0.2
- Upgraded `rand` from 0.8 to 0.9 (updated internal API usage)
- Upgraded `console` from 0.15 to 0.16
- Upgraded `dialoguer` from 0.11 to 0.12
- Upgraded `clap` from 4 to 4.5
- Upgraded `handlebars` from 6 to 6.4

### Fixed
- Updated `rand` API usage: `thread_rng()` → `rng()`, `gen_range()` → `random_range()`
- Fixed `brylix new` generated projects failing to build with `cargo lambda build --release --arm64` due to missing OpenSSL (now includes `openssl = { version = "0.10", features = ["vendored"] }`)
- Fixed `brylix new` to use correct sqlx feature for PostgreSQL projects (`sqlx-postgres` instead of `sqlx-mysql`)

## [0.1.3] - 2025-01-30

### Fixed
- Fixed `brylix dev` command to use `--invoke-port` instead of `--invoke-address` for cargo-lambda watch

## [0.1.2] - 2025-01-30

### Fixed
- Fixed `brylix dev` command failing with "unexpected argument '--port'" error by using `--invoke-address` flag for cargo-lambda watch

## [0.1.1] - 2025-01-30

### Added
- Added `brylix test` command for running tests

## [0.1.0] - 2025-01-30

### Added
- Initial framework extraction from bapi-manager
- Core `brylix` crate with:
  - Error handling system with domain and GraphQL error types
  - Configuration management with builder pattern
  - JWT authentication with tenant support
  - Password hashing using Argon2
  - Input validation utilities
  - Database initialization with MySQL/PostgreSQL support
  - Multi-tenant connection pooling (pool-per-droplet)
  - Lambda HTTP handler with CORS support
  - GraphQL context helpers and auth guards
  - Provider trait for external service integration
- `brylix-cli` crate with:
  - `brylix new` - Project scaffolding
  - `brylix generate` - Code generation (entity, service, repository, resolver, migration)
  - `brylix dev` - Local development with cargo-lambda
  - `brylix build` - Production builds for Lambda
  - `brylix deploy` - AWS Lambda deployment
  - `brylix migrate` - Database migration management
- Example projects:
  - `basic` - Single-tenant Blog API
  - `multi-tenant` - Multi-tenant Task SaaS
- Handlebars templates for code generation
- Comprehensive documentation (CLAUDE.md, README.md)

### Features
- MySQL support via `mysql` feature flag
- PostgreSQL support via `postgres` feature flag
- GraphQL Playground via `playground` feature flag
- Multi-tenant mode via `multi-tenant` feature flag

[Unreleased]: https://github.com/brylix/framework/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/brylix/framework/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/brylix/framework/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/brylix/framework/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/brylix/framework/compare/v0.1.3...v0.2.0
[0.1.3]: https://github.com/brylix/framework/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/brylix/framework/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/brylix/framework/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/brylix/framework/releases/tag/v0.1.0
