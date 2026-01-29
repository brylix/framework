//! Path routing utilities for multi-tenant applications.

/// Extract tenant name from path like /api/{tenant}
///
/// # Examples
///
/// ```rust
/// use brylix::handler::extract_tenant;
///
/// assert_eq!(extract_tenant("/api/acme"), Some("acme"));
/// assert_eq!(extract_tenant("/api/acme/graphql"), Some("acme"));
/// assert_eq!(extract_tenant("/api"), None);
/// assert_eq!(extract_tenant("/api/"), None);
/// ```
pub fn extract_tenant(path: &str) -> Option<&str> {
    path.strip_prefix("/api/")
        .and_then(|s| s.split('/').next())
        .filter(|s| !s.is_empty())
}

/// Extract tenant name from playground path like /playground/{tenant}
///
/// # Examples
///
/// ```rust
/// use brylix::handler::extract_playground_tenant;
///
/// assert_eq!(extract_playground_tenant("/playground/acme"), Some("acme"));
/// assert_eq!(extract_playground_tenant("/playground"), None);
/// ```
pub fn extract_playground_tenant(path: &str) -> Option<&str> {
    path.strip_prefix("/playground/")
        .and_then(|s| s.split('/').next())
        .filter(|s| !s.is_empty())
}

/// Check if a path is a GraphQL API endpoint.
pub fn is_api_path(path: &str) -> bool {
    path == "/api" || path.starts_with("/api/")
}

/// Check if a path is a GraphQL Playground endpoint.
pub fn is_playground_path(path: &str) -> bool {
    path == "/playground" || path.starts_with("/playground/")
}

/// Get the API endpoint for a playground request.
///
/// In multi-tenant mode, maps /playground/{tenant} to /api/{tenant}.
/// In single-tenant mode, maps /playground to /api.
pub fn playground_api_endpoint(path: &str, multi_tenant: bool) -> String {
    if multi_tenant {
        if let Some(tenant) = extract_playground_tenant(path) {
            return format!("/api/{}", tenant);
        }
    }
    "/api".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tenant() {
        assert_eq!(extract_tenant("/api/acme"), Some("acme"));
        assert_eq!(extract_tenant("/api/acme/graphql"), Some("acme"));
        assert_eq!(extract_tenant("/api/"), None);
        assert_eq!(extract_tenant("/api"), None);
        assert_eq!(extract_tenant("/other/path"), None);
    }

    #[test]
    fn test_extract_playground_tenant() {
        assert_eq!(extract_playground_tenant("/playground/acme"), Some("acme"));
        assert_eq!(extract_playground_tenant("/playground/"), None);
        assert_eq!(extract_playground_tenant("/playground"), None);
    }

    #[test]
    fn test_is_api_path() {
        assert!(is_api_path("/api"));
        assert!(is_api_path("/api/acme"));
        assert!(!is_api_path("/playground"));
        assert!(!is_api_path("/other"));
    }

    #[test]
    fn test_playground_api_endpoint() {
        assert_eq!(playground_api_endpoint("/playground/acme", true), "/api/acme");
        assert_eq!(playground_api_endpoint("/playground", true), "/api");
        assert_eq!(playground_api_endpoint("/playground/acme", false), "/api");
    }
}
