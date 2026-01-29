# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/brylix/framework/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/brylix/framework/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/brylix/framework/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/brylix/framework/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/brylix/framework/releases/tag/v0.1.0
