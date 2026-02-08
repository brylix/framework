//! GraphQL context data structures.

use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::auth::roles::AuthRole;

/// Information about the current tenant (multi-tenant mode).
#[derive(Clone, Debug)]
pub struct TenantInfo {
    /// Tenant name (also the database name)
    pub name: String,

    /// The database droplet ID hosting this tenant (for pool-per-droplet)
    pub droplet_id: Option<i64>,

    /// Current database schema version
    pub db_version: i32,
}

impl TenantInfo {
    /// Create a new TenantInfo
    pub fn new(name: impl Into<String>, droplet_id: Option<i64>, db_version: i32) -> Self {
        Self {
            name: name.into(),
            droplet_id,
            db_version,
        }
    }
}

/// Context data passed to all GraphQL resolvers.
///
/// This struct is injected into the GraphQL context and provides
/// access to the database connection, authenticated user, and tenant info.
///
/// # Usage
///
/// ```rust
/// use brylix::graphql::ContextData;
/// use async_graphql::Context;
///
/// async fn my_resolver(ctx: &Context<'_>) -> Result<User> {
///     let data = ctx.data_unchecked::<ContextData>();
///
///     // Access the database
///     let db = &data.db;
///
///     // Check if user is authenticated
///     if let Some(user_id) = &data.user {
///         // User is logged in
///     }
///
///     // Check tenant info (multi-tenant mode)
///     if let Some(tenant) = &data.tenant {
///         println!("Tenant: {}", tenant.name);
///     }
/// }
/// ```
pub struct ContextData {
    /// Database connection for the current request
    pub db: DatabaseConnection,

    /// Authenticated user ID (from JWT subject)
    /// None if the request is not authenticated
    pub user: Option<String>,

    /// Authentication role (user, admin, or custom)
    /// None if no role-based auth is configured
    pub role: Option<AuthRole>,

    /// Tenant information (multi-tenant mode only)
    /// Wrapped in Arc for shared ownership across resolvers
    pub tenant: Option<Arc<TenantInfo>>,
}

impl ContextData {
    /// Create a new ContextData
    pub fn new(
        db: DatabaseConnection,
        user: Option<String>,
        role: Option<AuthRole>,
        tenant: Option<TenantInfo>,
    ) -> Self {
        Self {
            db,
            user,
            role,
            tenant: tenant.map(Arc::new),
        }
    }

    /// Create a new ContextData for single-tenant mode
    pub fn single_tenant(
        db: DatabaseConnection,
        user: Option<String>,
        role: Option<AuthRole>,
    ) -> Self {
        Self {
            db,
            user,
            role,
            tenant: None,
        }
    }

    /// Create a new ContextData with tenant info
    pub fn multi_tenant(
        db: DatabaseConnection,
        user: Option<String>,
        role: Option<AuthRole>,
        tenant: TenantInfo,
    ) -> Self {
        Self {
            db,
            user,
            role,
            tenant: Some(Arc::new(tenant)),
        }
    }

    /// Check if the request is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.user.is_some()
    }

    /// Get the user ID if authenticated
    pub fn user_id(&self) -> Option<&str> {
        self.user.as_deref()
    }

    /// Get the authentication role if set
    pub fn auth_role(&self) -> Option<&AuthRole> {
        self.role.as_ref()
    }

    /// Check if the authenticated user is an admin
    pub fn is_admin(&self) -> bool {
        self.role.as_ref().is_some_and(|r| r.is_admin())
    }

    /// Get the tenant name if in multi-tenant mode
    pub fn tenant_name(&self) -> Option<&str> {
        self.tenant.as_ref().map(|t| t.name.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_info() {
        let info = TenantInfo::new("acme", Some(1), 2);
        assert_eq!(info.name, "acme");
        assert_eq!(info.droplet_id, Some(1));
        assert_eq!(info.db_version, 2);
    }
}
