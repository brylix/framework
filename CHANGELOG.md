# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.8] - 2026-02-08

### Added
- **Admin Override** (`auth::admin_override`, feature: `admin-override`) - Temporary admin elevation for POS/kiosk scenarios where an admin "taps in" to authorize a single privileged action without logging out the current user
  - `AdminOverrideConfig` - Configuration with secret and expiry (default 60 seconds)
  - `AdminOverrideClaims` - JWT claims with `token_type: "admin_override"` marker to prevent regular admin JWTs from being used as overrides
  - `AdminOverride` - Validated result containing admin ID, name, and optional action
  - `issue_admin_override_token()` - Issue short-lived override tokens after admin credential verification
  - `validate_admin_override_token()` - Validate and decode override tokens
  - `extract_admin_override_header()` - Extract `X-Admin-Override` header from requests
  - `get_admin_override()` - Get override info from GraphQL context
  - `require_auth_with_admin_override()` - Guard requiring both user auth and admin override
  - `AdminOverrideAudit` - Audit trail struct with `.log()` method for tracing
  - `admin_override_middleware()` - Middleware for extracting and validating override tokens from requests
  - `ADMIN_OVERRIDE_HEADER` constant
  - All types exported via prelude when feature is enabled
- New environment variables:
  - `ADMIN_JWT_SECRET` - Secret for admin override tokens (reuses admin role secret)
  - `ADMIN_OVERRIDE_EXPIRY_SECS` - Token expiry in seconds (optional, default: 60)

### Changed
- **BREAKING**: `ContextData::new()`, `single_tenant()`, and `multi_tenant()` now accept an additional `admin_override: Option<AdminOverride>` parameter when `admin-override` feature is enabled
  - Migration: add `None` as the admin_override argument when using the `admin-override` feature
  - Example: `ContextData::single_tenant(db, user, role, None)` (with feature enabled)
  - Without the feature enabled, existing code continues to work unchanged
- `require_admin(ctx)` now also checks for admin override — returns the admin ID from the override when present
- `ContextData::is_admin()` now returns `true` when an admin override is present
- CORS headers now include `X-Admin-Override` in `Access-Control-Allow-Headers`
- Updated `full` feature to include `admin-override`
- Added `admin_override: Option<AdminOverrideConfig>` to `Config` struct (feature-gated)
- Added `admin_override_secret()` and `admin_override_expiry_secs()` builder methods to `ConfigBuilder`

## [0.2.7] - 2026-02-08

### Added
- **Pagination Utilities** (`graphql::pagination`) - Generic `PageInfo`, `Connection<T>`, `IntoConnection` trait, and `page_info()` function for building paginated GraphQL responses
- **GraphQL ID Parsing** (`graphql::helpers`) - `parse_gql_id()`, `parse_gql_id_field()`, and `gql_id!` macro for parsing string IDs to i64
- **JSON Column Helpers** (`helpers::json`) - `JsonValueExt` trait with `parse_as()` and `parse_or_default()` for ergonomic `serde_json::Value` parsing
- **Timestamp Helpers** (`helpers::time`) - `utc_now()` function and `Timestamped` trait for managing `created_at`/`updated_at` fields
- **Soft Delete Pattern** (`helpers::soft_delete`) - `SoftDeletable` trait and `status` module with common lifecycle constants (active, pending, approved, rejected, deleted)
- **Multi-Role Authentication** (`auth::roles`) - `AuthRole` enum (User, Admin, Custom), `MultiRoleJwtConfig` for multiple JWT secrets, `require_admin()` and `get_auth_role()` guards
- New `helpers` module with `json`, `time`, and `soft_delete` submodules
- All new types and functions exported via prelude

### Changed
- **BREAKING**: `ContextData::new()`, `single_tenant()`, and `multi_tenant()` now require a `role: Option<AuthRole>` parameter
  - Migration: add `None` as the role argument for projects not using multi-role auth
  - Example: `ContextData::single_tenant(db, user, None)`
- Added `role: Option<AuthRole>` field to `ContextData` struct
- Added `auth_role()` and `is_admin()` helper methods to `ContextData`

## [0.2.6] - 2026-02-04

### Added
- Custom S3 credentials support for local development via `S3_ACCESS_KEY_ID` and `S3_SECRET_ACCESS_KEY` environment variables
- Automatic fallback to default AWS credential chain (IAM role) when custom credentials are not set - ideal for Lambda deployments
- New `with_max_size(bytes)` builder method on `PresignedUrlRequest` for specifying maximum allowed file size
- New `max_size_bytes` field in `PresignedUrlResponse` for client-side validation
- New dependency: `aws-credential-types` for custom credential provider support

### Changed
- `AwsS3Provider::new()` now checks for custom S3 credentials before falling back to default AWS credential chain

## [0.2.5] - 2026-02-04

### Fixed
- Updated crate README.md with S3 feature documentation

## [0.2.4] - 2026-02-04

### Added
- New `s3` feature for S3 presigned URL generation
- `S3Provider` trait for custom S3 provider implementations
- `AwsS3Provider` - AWS S3 implementation using aws-sdk-s3
- `PresignedUrlRequest` - Builder for constructing upload requests with content type and expiration
- `PresignedUrlResponse` - Response containing presigned URL, method, key, and expiration timestamp
- `NoOpS3Provider` - Testing/mock S3 provider
- Multi-tenant support with automatic tenant-prefixed paths (`/{tenant}/{folder}/{filename}`)
- New environment variables for S3 configuration:
  - `S3_BUCKET` - S3 bucket name (required)
  - `S3_REGION` - AWS region (default: us-east-1)
  - `S3_UPLOAD_EXPIRES_SECS` - Upload URL expiration in seconds (default: 3600)
  - `S3_DOWNLOAD_EXPIRES_SECS` - Download URL expiration in seconds (default: 3600)
- New workspace dependencies: `aws-config`, `aws-sdk-s3`
- Documentation: File Uploads (S3) guide

### Changed
- Updated `full` feature to include `s3`

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

[Unreleased]: https://github.com/brylix/framework/compare/v0.2.8...HEAD
[0.2.8]: https://github.com/brylix/framework/compare/v0.2.7...v0.2.8
[0.2.7]: https://github.com/brylix/framework/compare/v0.2.6...v0.2.7
[0.2.6]: https://github.com/brylix/framework/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/brylix/framework/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/brylix/framework/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/brylix/framework/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/brylix/framework/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/brylix/framework/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/brylix/framework/compare/v0.1.3...v0.2.0
[0.1.3]: https://github.com/brylix/framework/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/brylix/framework/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/brylix/framework/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/brylix/framework/releases/tag/v0.1.0
