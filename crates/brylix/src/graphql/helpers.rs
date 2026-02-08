//! GraphQL helper functions for common operations.
//!
//! Provides utilities for parsing GraphQL IDs and other common patterns.
//!
//! # Usage
//!
//! ```rust
//! use brylix::graphql::helpers::parse_gql_id;
//!
//! // In a resolver:
//! // let user_id = parse_gql_id(&input.user_id)?;
//! ```

use async_graphql::Error as GqlError;

use crate::errors::gql_bad_request;

/// Parse a GraphQL String ID to i64.
///
/// GraphQL often represents IDs as strings. This helper parses them
/// to i64 with a descriptive error message.
///
/// # Arguments
///
/// * `id` - The string ID to parse
///
/// # Returns
///
/// The parsed i64 value
///
/// # Errors
///
/// Returns a BAD_REQUEST GraphQL error if the ID cannot be parsed
///
/// # Example
///
/// ```rust,ignore
/// use brylix::graphql::helpers::parse_gql_id;
///
/// let id = parse_gql_id("123")?; // Ok(123)
/// let err = parse_gql_id("abc");  // Err(BAD_REQUEST)
/// ```
pub fn parse_gql_id(id: &str) -> Result<i64, GqlError> {
    id.parse::<i64>()
        .map_err(|_| gql_bad_request(format!("Invalid ID: {}", id)))
}

/// Parse a GraphQL String ID to i64 with a custom field name in the error.
///
/// # Arguments
///
/// * `id` - The string ID to parse
/// * `field` - The field name to include in the error message
///
/// # Returns
///
/// The parsed i64 value
///
/// # Errors
///
/// Returns a BAD_REQUEST GraphQL error with the field name
///
/// # Example
///
/// ```rust,ignore
/// use brylix::graphql::helpers::parse_gql_id_field;
///
/// let id = parse_gql_id_field("123", "user_id")?; // Ok(123)
/// // Error: "Invalid user_id: abc"
/// ```
pub fn parse_gql_id_field(id: &str, field: &str) -> Result<i64, GqlError> {
    id.parse::<i64>()
        .map_err(|_| gql_bad_request(format!("Invalid {}: {}", field, id)))
}

/// Convenience macro for parsing GraphQL IDs.
///
/// # Usage
///
/// ```rust,ignore
/// use brylix::gql_id;
///
/// // Parse with default error message
/// let id = gql_id!("123");
///
/// // Parse with custom field name
/// let id = gql_id!("123", "user_id");
/// ```
#[macro_export]
macro_rules! gql_id {
    ($id:expr) => {
        $crate::graphql::helpers::parse_gql_id($id)?
    };
    ($id:expr, $field:expr) => {
        $crate::graphql::helpers::parse_gql_id_field($id, $field)?
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gql_id_valid() {
        assert_eq!(parse_gql_id("123").unwrap(), 123);
        assert_eq!(parse_gql_id("0").unwrap(), 0);
        assert_eq!(parse_gql_id("-1").unwrap(), -1);
    }

    #[test]
    fn test_parse_gql_id_invalid() {
        assert!(parse_gql_id("abc").is_err());
        assert!(parse_gql_id("").is_err());
        assert!(parse_gql_id("12.34").is_err());
    }

    #[test]
    fn test_parse_gql_id_field_valid() {
        assert_eq!(parse_gql_id_field("42", "user_id").unwrap(), 42);
    }

    #[test]
    fn test_parse_gql_id_field_invalid() {
        let err = parse_gql_id_field("abc", "user_id").unwrap_err();
        assert!(err.message.contains("user_id"));
    }
}
