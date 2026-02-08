//! Prelude module for convenient imports.
//!
//! Import everything commonly needed with:
//!
//! ```rust
//! use brylix::prelude::*;
//! ```

// Error handling
pub use crate::errors::{
    gql_bad_request, gql_error, gql_from_anyhow, gql_from_domain, gql_internal, gql_not_found,
    gql_unauthorized, DomainError, DomainResult,
};

// Configuration
pub use crate::config::{Config, ConfigBuilder};

// Authentication
pub use crate::auth::{issue_jwt, jwt_middleware, validate_jwt, Claims, JwtResult};
pub use crate::auth::{hash_password, verify_password};

// Validation
pub use crate::validation::{
    validate_email, validate_hostname, validate_name, validate_password, validate_tenant_name,
};

// GraphQL
pub use crate::graphql::{require_auth, require_auth_user_id, ContextData};

// Pagination
pub use crate::graphql::pagination::{page_info, Connection, IntoConnection, PageInfo};

// ID Parsing
pub use crate::graphql::helpers::{parse_gql_id, parse_gql_id_field};

// Helpers
pub use crate::helpers::json::JsonValueExt;
pub use crate::helpers::soft_delete::{status, SoftDeletable};
pub use crate::helpers::time::{utc_now, Timestamped};

// Multi-role Auth
pub use crate::auth::roles::{get_auth_role, require_admin, AuthRole, MultiRoleJwtConfig};

// Database
pub use crate::db::{connect, connect_with_pool, init_db, PoolConfig};

// Handler
pub use crate::handler::{
    cors_preflight, error_response, graphql_error, graphql_request_from_get,
    graphql_request_from_post, graphql_response, playground_response,
};

// Provider
pub use crate::provider::{HealthMetrics, Provider, ResourceInfo};

// Email Provider (feature-gated)
#[cfg(feature = "email")]
pub use crate::provider::email::{
    EmailAttachment, EmailMessage, EmailProvider, NoOpEmailProvider, SmtpProvider,
};

// S3 Provider (feature-gated)
#[cfg(feature = "s3")]
pub use crate::provider::s3::{
    AwsS3Provider, NoOpS3Provider, PresignedUrlRequest, PresignedUrlResponse, S3Provider,
};

// Multi-tenant (feature-gated)
#[cfg(feature = "multi-tenant")]
pub use crate::tenant::{get_tenant_manager, TenantError, TenantInfo, TenantManager};

// Re-exports from dependencies for convenience
pub use async_graphql::{Context, EmptySubscription, Object, Result as GqlResult, Schema, SimpleObject};
pub use sea_orm::DatabaseConnection;
pub use lambda_http::{Body, Request, Response};
