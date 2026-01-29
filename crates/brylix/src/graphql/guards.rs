//! Authentication guards for GraphQL resolvers.

use async_graphql::{Context, Result};

use super::ContextData;
use crate::errors::{gql_bad_request, gql_unauthorized};

/// Require authentication and return the JWT subject (typically user_id as string).
///
/// # Arguments
///
/// * `ctx` - The GraphQL context
///
/// # Returns
///
/// The JWT subject string (user ID)
///
/// # Errors
///
/// Returns UNAUTHORIZED error if not authenticated
///
/// # Example
///
/// ```rust
/// use brylix::graphql::require_auth;
///
/// async fn protected_resolver(ctx: &Context<'_>) -> Result<String> {
///     let user_id_str = require_auth(ctx)?;
///     Ok(format!("Hello, user {}", user_id_str))
/// }
/// ```
pub fn require_auth(ctx: &Context<'_>) -> Result<String> {
    let data = ctx.data_unchecked::<ContextData>();
    match &data.user {
        Some(sub) => Ok(sub.clone()),
        None => Err(gql_unauthorized()),
    }
}

/// Require authentication and return the user_id as i64.
///
/// This is a convenience function that combines `require_auth` with parsing
/// the JWT subject into a user ID.
///
/// # Arguments
///
/// * `ctx` - The GraphQL context
///
/// # Returns
///
/// The user ID as i64
///
/// # Errors
///
/// Returns UNAUTHORIZED if not authenticated, or BAD_REQUEST if the
/// user ID cannot be parsed as an integer.
///
/// # Example
///
/// ```rust
/// use brylix::graphql::require_auth_user_id;
///
/// async fn get_my_profile(ctx: &Context<'_>) -> Result<User> {
///     let user_id = require_auth_user_id(ctx)?;
///     UserService::get_by_id(db, user_id).await
/// }
/// ```
pub fn require_auth_user_id(ctx: &Context<'_>) -> Result<i64> {
    let sub = require_auth(ctx)?;
    sub.parse()
        .map_err(|_| gql_bad_request("Invalid user id in token"))
}

/// Check if the current request is authenticated.
///
/// # Arguments
///
/// * `ctx` - The GraphQL context
///
/// # Returns
///
/// `true` if authenticated, `false` otherwise
pub fn is_authenticated(ctx: &Context<'_>) -> bool {
    let data = ctx.data_unchecked::<ContextData>();
    data.is_authenticated()
}

/// Get the optional user ID without requiring authentication.
///
/// # Arguments
///
/// * `ctx` - The GraphQL context
///
/// # Returns
///
/// The user ID string if authenticated, None otherwise
pub fn get_user_id(ctx: &Context<'_>) -> Option<String> {
    let data = ctx.data_unchecked::<ContextData>();
    data.user.clone()
}

/// Get the optional tenant info.
///
/// # Arguments
///
/// * `ctx` - The GraphQL context
///
/// # Returns
///
/// A reference to the TenantInfo if in multi-tenant mode
#[cfg(feature = "multi-tenant")]
pub fn get_tenant<'a>(ctx: &'a Context<'a>) -> Option<&'a super::TenantInfo> {
    let data = ctx.data_unchecked::<ContextData>();
    data.tenant.as_ref().map(|t| t.as_ref())
}

#[cfg(test)]
mod tests {
    // Note: These tests would require mocking the GraphQL context
    // which is complex. Integration tests are more appropriate.
}
