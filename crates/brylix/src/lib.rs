//! # Brylix Framework
//!
//! A Rust framework for building GraphQL APIs on AWS Lambda with SeaORM
//! and multi-tenant support.
//!
//! ## Features
//!
//! - **GraphQL API** - Built on async-graphql with playground support
//! - **AWS Lambda** - Optimized for serverless deployment
//! - **SeaORM** - Type-safe database operations
//! - **Multi-tenant** - Pool-per-droplet architecture for SaaS
//! - **JWT Authentication** - Secure token-based auth
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use brylix::prelude::*;
//! use async_graphql::{EmptySubscription, Schema};
//!
//! mod graphql;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), brylix::Error> {
//!     Brylix::builder()
//!         .config_from_env()
//!         .with_jwt_auth()
//!         .with_migrations::<migration::Migrator>()
//!         .build_schema(|| {
//!             Schema::build(graphql::Query, graphql::Mutation, EmptySubscription).finish()
//!         })
//!         .build()?
//!         .run()
//!         .await
//! }
//! ```
//!
//! ## Module Overview
//!
//! - [`errors`] - Error handling with GraphQL integration
//! - [`config`] - Configuration management
//! - [`auth`] - JWT authentication
//! - [`validation`] - Input validation
//! - [`db`] - Database initialization
//! - [`graphql`] - GraphQL context and guards
//! - [`handler`] - Lambda HTTP handling
//! - [`provider`] - External service integration
//! - [`provider::email`] - Email provider (feature: `email`)
//! - [`tenant`] - Multi-tenant connection management (feature: `multi-tenant`)

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod auth;
pub mod config;
pub mod db;
pub mod errors;
pub mod graphql;
pub mod handler;
pub mod provider;
pub mod validation;

#[cfg(feature = "multi-tenant")]
pub mod tenant;

pub mod prelude;

// Re-export commonly used types at the crate root
pub use errors::{DomainError, DomainResult};
pub use config::Config;
pub use graphql::ContextData;

/// Lambda error type alias
pub type Error = lambda_http::Error;

/// Result type for Lambda handlers
pub type Result<T> = std::result::Result<T, Error>;
