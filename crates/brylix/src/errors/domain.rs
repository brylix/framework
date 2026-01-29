//! Domain error types for business logic errors.

use thiserror::Error;

/// Domain-specific errors with proper error codes for GraphQL responses.
///
/// Each variant maps to a specific GraphQL error code. Applications can
/// extend this with their own error types by wrapping or using the generic
/// variants like `NotFound`, `InvalidInput`, etc.
#[derive(Error, Debug)]
pub enum DomainError {
    // ============================================================================
    // Authentication & Authorization errors
    // ============================================================================
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Invalid user id in token")]
    InvalidTokenUserId,

    // ============================================================================
    // Not found errors
    // ============================================================================
    #[error("User not found")]
    UserNotFound,

    #[error("User with id {0} not found")]
    UserNotFoundById(i64),

    #[error("Not found: {0}")]
    NotFound(String),

    // ============================================================================
    // Configuration & Service errors
    // ============================================================================
    #[error("Config not initialized")]
    ConfigNotInitialized,

    #[error("Provider not configured")]
    ProviderNotConfigured,

    // ============================================================================
    // Validation errors
    // ============================================================================
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    // ============================================================================
    // Database errors
    // ============================================================================
    #[error("Database error: {0}")]
    DatabaseError(String),

    // ============================================================================
    // External service errors
    // ============================================================================
    #[error("External service error: {0}")]
    ExternalService(String),

    // ============================================================================
    // JWT errors
    // ============================================================================
    #[error("Failed to issue token: {0}")]
    TokenIssueFailed(String),

    // ============================================================================
    // Generic internal error
    // ============================================================================
    #[error("Internal error: {0}")]
    Internal(String),

    // ============================================================================
    // Tenant errors (multi-tenant mode)
    // ============================================================================
    #[error("Invalid tenant name: {0}")]
    TenantInvalid(String),

    #[error("Tenant not found: {0}")]
    TenantNotFound(String),

    #[error("Tenant mismatch: JWT tenant does not match URL tenant")]
    TenantMismatch,

    #[error("Upgrade required: feature '{feature}' requires database version {required_version} (current: {current_version})")]
    UpgradeRequired {
        current_version: i32,
        required_version: i32,
        feature: String,
    },
}

impl DomainError {
    /// Get the GraphQL error code for this error
    pub fn code(&self) -> &'static str {
        match self {
            // Authentication
            DomainError::InvalidCredentials => "INVALID_CREDENTIALS",
            DomainError::Unauthorized => "UNAUTHORIZED",
            DomainError::Forbidden(_) => "FORBIDDEN",
            DomainError::InvalidTokenUserId => "BAD_REQUEST",

            // Not found
            DomainError::UserNotFound => "NOT_FOUND",
            DomainError::UserNotFoundById(_) => "NOT_FOUND",
            DomainError::NotFound(_) => "NOT_FOUND",

            // Configuration
            DomainError::ConfigNotInitialized => "SERVICE_UNAVAILABLE",
            DomainError::ProviderNotConfigured => "SERVICE_UNAVAILABLE",

            // Validation
            DomainError::InvalidInput(_) => "BAD_REQUEST",
            DomainError::DuplicateEntry(_) => "CONFLICT",

            // Database
            DomainError::DatabaseError(_) => "INTERNAL",

            // External
            DomainError::ExternalService(_) => "BAD_GATEWAY",

            // JWT
            DomainError::TokenIssueFailed(_) => "INTERNAL",

            // Generic
            DomainError::Internal(_) => "INTERNAL",

            // Tenant
            DomainError::TenantInvalid(_) => "BAD_REQUEST",
            DomainError::TenantNotFound(_) => "NOT_FOUND",
            DomainError::TenantMismatch => "FORBIDDEN",
            DomainError::UpgradeRequired { .. } => "UPGRADE_REQUIRED",
        }
    }

    /// Check if this is a not-found error
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            DomainError::UserNotFound
                | DomainError::UserNotFoundById(_)
                | DomainError::NotFound(_)
                | DomainError::TenantNotFound(_)
        )
    }

    /// Check if this is an auth error
    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            DomainError::InvalidCredentials
                | DomainError::Unauthorized
                | DomainError::Forbidden(_)
                | DomainError::TenantMismatch
        )
    }

    /// Check if this is a validation error
    pub fn is_validation_error(&self) -> bool {
        matches!(
            self,
            DomainError::InvalidInput(_)
                | DomainError::DuplicateEntry(_)
                | DomainError::TenantInvalid(_)
        )
    }
}

/// Result type for domain operations
pub type DomainResult<T> = Result<T, DomainError>;

// ============================================================================
// Conversion helpers for common error sources
// ============================================================================

impl From<sea_orm::DbErr> for DomainError {
    fn from(err: sea_orm::DbErr) -> Self {
        let msg = err.to_string();
        // Check for duplicate entry errors (SeaORM usually wraps SQL errors)
        if msg.contains("Duplicate entry") || msg.contains("1062") {
            DomainError::DuplicateEntry(msg)
        } else {
            DomainError::DatabaseError(msg)
        }
    }
}

impl From<anyhow::Error> for DomainError {
    fn from(err: anyhow::Error) -> Self {
        // Try to downcast to DomainError first
        match err.downcast::<DomainError>() {
            Ok(domain_err) => domain_err,
            Err(err) => DomainError::Internal(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(DomainError::InvalidCredentials.code(), "INVALID_CREDENTIALS");
        assert_eq!(DomainError::Unauthorized.code(), "UNAUTHORIZED");
        assert_eq!(DomainError::NotFound("test".into()).code(), "NOT_FOUND");
        assert_eq!(DomainError::InvalidInput("test".into()).code(), "BAD_REQUEST");
    }

    #[test]
    fn test_is_not_found() {
        assert!(DomainError::UserNotFound.is_not_found());
        assert!(DomainError::NotFound("test".into()).is_not_found());
        assert!(!DomainError::InvalidCredentials.is_not_found());
    }

    #[test]
    fn test_is_auth_error() {
        assert!(DomainError::InvalidCredentials.is_auth_error());
        assert!(DomainError::Unauthorized.is_auth_error());
        assert!(DomainError::Forbidden("test".into()).is_auth_error());
        assert!(!DomainError::NotFound("test".into()).is_auth_error());
    }
}
