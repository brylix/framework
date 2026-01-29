//! Error handling module for the Brylix framework.
//!
//! Provides type-safe error handling with proper GraphQL error codes.
//!
//! # Usage
//!
//! ```rust
//! use brylix::errors::{DomainError, DomainResult, gql_from_domain};
//!
//! fn get_user(id: i64) -> DomainResult<User> {
//!     users.find_by_id(id)
//!         .ok_or(DomainError::NotFound(format!("User {} not found", id)))
//! }
//!
//! // In GraphQL resolver:
//! let user = get_user(id).map_err(gql_from_domain)?;
//! ```

mod domain;
mod graphql;
mod http;

pub use domain::{DomainError, DomainResult};
pub use graphql::{
    gql_bad_request, gql_error, gql_from_anyhow, gql_from_domain, gql_from_message,
    gql_internal, gql_not_found, gql_tenant_invalid, gql_tenant_mismatch,
    gql_tenant_not_found, gql_unauthorized, gql_upgrade_required,
};
pub use http::{ClientError, ServerError};
