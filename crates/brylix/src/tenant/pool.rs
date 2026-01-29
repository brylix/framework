//! Tenant connection pool types and errors.

use thiserror::Error;

/// Information about a tenant for connection management.
#[derive(Clone, Debug)]
pub struct TenantInfo {
    /// Tenant name (also the database name)
    pub name: String,

    /// The database droplet ID hosting this tenant
    pub droplet_id: Option<i64>,

    /// Current database schema version
    pub db_version: i32,
}

impl TenantInfo {
    /// Create new tenant info
    pub fn new(name: impl Into<String>, droplet_id: Option<i64>, db_version: i32) -> Self {
        Self {
            name: name.into(),
            droplet_id,
            db_version,
        }
    }
}

/// Error type for tenant operations.
#[derive(Debug, Error)]
pub enum TenantError {
    #[error("Invalid tenant name: {0}")]
    InvalidName(String),

    #[error("Tenant not found: {0}")]
    NotFound(String),

    #[error("Droplet not found: {0}")]
    DropletNotFound(i64),

    #[error("Invalid droplet type: expected DATABASE MASTER")]
    InvalidDropletType,

    #[error("Tenant has no assigned droplet")]
    NoDropletAssigned,

    #[error("Tenant mismatch")]
    Mismatch,

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<TenantError> for crate::errors::DomainError {
    fn from(err: TenantError) -> Self {
        match err {
            TenantError::InvalidName(name) => crate::errors::DomainError::TenantInvalid(name),
            TenantError::NotFound(name) => crate::errors::DomainError::TenantNotFound(name),
            TenantError::DropletNotFound(id) => {
                crate::errors::DomainError::Internal(format!("Droplet not found: {}", id))
            }
            TenantError::InvalidDropletType => crate::errors::DomainError::Internal(
                "Invalid droplet type: expected DATABASE MASTER".to_string(),
            ),
            TenantError::NoDropletAssigned => {
                crate::errors::DomainError::Internal("Tenant has no assigned droplet".to_string())
            }
            TenantError::Mismatch => crate::errors::DomainError::TenantMismatch,
            TenantError::Database(e) => crate::errors::DomainError::DatabaseError(e.to_string()),
            TenantError::Internal(msg) => crate::errors::DomainError::Internal(msg),
        }
    }
}
