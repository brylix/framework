//! Multi-tenant database connection management.
//!
//! Provides pool-per-droplet architecture for efficient connection management
//! across multiple tenant databases.
//!
//! # Architecture
//!
//! - Tenants are distributed across multiple database server droplets
//! - Connection pools are cached by `droplet_id`, not by tenant
//! - `USE {tenant_db}` switches database context per request (~1ms)
//!
//! This reduces memory from O(tenants) to O(database_droplets).
//!
//! # Usage
//!
//! ```rust
//! use brylix::tenant::{TenantManager, TenantInfo};
//!
//! // Get the global tenant manager
//! let manager = get_tenant_manager().await;
//!
//! // Get connection for a tenant
//! let (db, info) = manager.get_connection("acme").await?;
//!
//! // Use the connection
//! let users = User::find().all(&db).await?;
//! ```

mod manager;
mod pool;

pub use manager::{get_tenant_manager, TenantManager};
pub use pool::{TenantError, TenantInfo};

// Re-export for convenience
pub use crate::graphql::TenantInfo as ContextTenantInfo;
