//! GraphQL error helpers for converting domain errors to GraphQL errors.

use async_graphql::{Error as GqlError, ErrorExtensions};

use super::DomainError;

/// Create a GraphQL error with a code extension
pub fn gql_error(code: &'static str, message: impl Into<String>) -> GqlError {
    let msg = message.into();
    GqlError::new(msg).extend_with(|_, e| e.set("code", code))
}

/// Create an internal error
pub fn gql_internal(message: impl Into<String>) -> GqlError {
    gql_error("INTERNAL", message)
}

/// Create a bad request error
pub fn gql_bad_request(message: impl Into<String>) -> GqlError {
    gql_error("BAD_REQUEST", message)
}

/// Create an unauthorized error
pub fn gql_unauthorized() -> GqlError {
    gql_error("UNAUTHORIZED", "Unauthorized")
}

/// Create a not found error
pub fn gql_not_found(message: impl Into<String>) -> GqlError {
    gql_error("NOT_FOUND", message)
}

/// Create a tenant invalid error
pub fn gql_tenant_invalid(tenant: &str) -> GqlError {
    gql_error("BAD_REQUEST", format!("Invalid tenant name: {}", tenant))
}

/// Create a tenant not found error
pub fn gql_tenant_not_found(tenant: &str) -> GqlError {
    gql_error("NOT_FOUND", format!("Tenant not found: {}", tenant))
}

/// Create a tenant mismatch error
pub fn gql_tenant_mismatch() -> GqlError {
    gql_error(
        "FORBIDDEN",
        "Tenant mismatch: JWT tenant does not match URL tenant",
    )
}

/// Create an upgrade required error
pub fn gql_upgrade_required(feature: &str, current: i32, required: i32) -> GqlError {
    gql_error(
        "UPGRADE_REQUIRED",
        format!(
            "Feature '{}' requires database version {} (current: {}). Please upgrade.",
            feature, required, current
        ),
    )
}

/// Convert anyhow::Error to GraphQL error.
/// First tries to downcast to DomainError for proper error codes,
/// then falls back to string classification for legacy errors.
pub fn gql_from_anyhow(err: anyhow::Error) -> GqlError {
    // Try to downcast to DomainError first
    if let Some(domain_err) = err.downcast_ref::<DomainError>() {
        return gql_error(domain_err.code(), domain_err.to_string());
    }

    // Fallback to string-based classification for legacy errors
    let msg = err.to_string();
    let code = classify_message_code(&msg);
    gql_error(code, msg)
}

/// Convert a message string to a GraphQL error with classified code
pub fn gql_from_message(message: String) -> GqlError {
    let code = classify_message_code(&message);
    gql_error(code, message)
}

/// Legacy string-based classifier for backwards compatibility.
/// New code should use DomainError variants instead.
fn classify_message_code(msg: &str) -> &'static str {
    if msg.contains("Invalid credentials") {
        "INVALID_CREDENTIALS"
    } else if msg.contains("not found") || msg.contains("Not found") {
        "NOT_FOUND"
    } else if msg.contains("Provider not configured") || msg.contains("Config not initialized") {
        "SERVICE_UNAVAILABLE"
    } else if msg.contains("Invalid user id in token") || msg.contains("Invalid") {
        "BAD_REQUEST"
    } else if msg.contains("FORBIDDEN") || msg.contains("Forbidden") {
        "FORBIDDEN"
    } else {
        "INTERNAL"
    }
}

/// Convert DomainError directly to GraphQL error.
/// Use this when the service/repository returns DomainResult.
pub fn gql_from_domain(err: DomainError) -> GqlError {
    gql_error(err.code(), err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gql_error() {
        let err = gql_error("TEST_CODE", "Test message");
        assert_eq!(err.message, "Test message");
    }

    #[test]
    fn test_gql_from_domain() {
        let domain_err = DomainError::NotFound("User".into());
        let gql_err = gql_from_domain(domain_err);
        assert!(gql_err.message.contains("User"));
    }

    #[test]
    fn test_classify_message_code() {
        assert_eq!(classify_message_code("User not found"), "NOT_FOUND");
        assert_eq!(classify_message_code("Invalid credentials"), "INVALID_CREDENTIALS");
        assert_eq!(classify_message_code("Something went wrong"), "INTERNAL");
    }
}
