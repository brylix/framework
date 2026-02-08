//! GraphQL utilities for Brylix applications.
//!
//! Provides context management and authentication guards for GraphQL resolvers.
//!
//! # Usage
//!
//! ```rust
//! use brylix::graphql::{ContextData, require_auth, require_auth_user_id};
//! use async_graphql::{Context, Object, Result};
//!
//! pub struct Query;
//!
//! #[Object]
//! impl Query {
//!     async fn me(&self, ctx: &Context<'_>) -> Result<User> {
//!         let user_id = require_auth_user_id(ctx)?;
//!         let data = ctx.data_unchecked::<ContextData>();
//!
//!         UserService::get_by_id(&data.db, user_id).await
//!     }
//! }
//! ```

mod context;
mod guards;

/// GraphQL helper functions (ID parsing, etc.)
pub mod helpers;

/// Pagination utilities for GraphQL connections.
pub mod pagination;

pub use context::ContextData;
pub use guards::{require_auth, require_auth_user_id};

#[cfg(feature = "multi-tenant")]
pub use context::TenantInfo;
