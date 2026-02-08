//! Multi-role authentication support.
//!
//! Provides role-based authentication with support for multiple JWT secrets,
//! allowing different token types (user, admin, custom) to coexist.
//!
//! # Usage
//!
//! ```rust,ignore
//! use brylix::auth::roles::{AuthRole, MultiRoleJwtConfig};
//!
//! // Configure multiple JWT secrets
//! let jwt_config = MultiRoleJwtConfig::new()
//!     .add_role("user", std::env::var("JWT_SECRET").unwrap())
//!     .add_role("admin", std::env::var("ADMIN_JWT_SECRET").unwrap());
//!
//! // Validate a token against all configured secrets
//! if let Some(role) = jwt_config.validate(token) {
//!     match &role {
//!         AuthRole::Admin(id) => println!("Admin: {}", id),
//!         AuthRole::User(id) => println!("User: {}", id),
//!         AuthRole::Custom(name, id) => println!("{}: {}", name, id),
//!     }
//! }
//! ```

use async_graphql::{Context, Error as GqlError};
use jsonwebtoken::{decode, DecodingKey, Validation};

use super::Claims;
use crate::errors::{gql_error, gql_unauthorized};
use crate::graphql::ContextData;

/// Authentication role representing the type and identity of an authenticated user.
#[derive(Debug, Clone, PartialEq)]
pub enum AuthRole {
    /// Regular user with their user ID
    User(i64),

    /// Admin user with their user ID
    Admin(i64),

    /// Custom role with a role name and user ID
    Custom(String, i64),
}

impl AuthRole {
    /// Get the user ID regardless of role type.
    pub fn id(&self) -> i64 {
        match self {
            AuthRole::User(id) => *id,
            AuthRole::Admin(id) => *id,
            AuthRole::Custom(_, id) => *id,
        }
    }

    /// Check if this is an admin role.
    pub fn is_admin(&self) -> bool {
        matches!(self, AuthRole::Admin(_))
    }

    /// Check if this is a regular user role.
    pub fn is_user(&self) -> bool {
        matches!(self, AuthRole::User(_))
    }

    /// Get the role name as a string.
    pub fn role_name(&self) -> &str {
        match self {
            AuthRole::User(_) => "user",
            AuthRole::Admin(_) => "admin",
            AuthRole::Custom(name, _) => name,
        }
    }
}

/// Multi-secret JWT configuration for role-based authentication.
///
/// Each role has its own JWT secret, allowing different token types
/// to be issued and validated independently.
///
/// # Example
///
/// ```rust,ignore
/// use brylix::auth::roles::MultiRoleJwtConfig;
///
/// let config = MultiRoleJwtConfig::new()
///     .add_role("user", user_secret)
///     .add_role("admin", admin_secret);
///
/// // Returns the first role whose secret successfully validates the token
/// let role = config.validate(token);
/// ```
pub struct MultiRoleJwtConfig {
    secrets: Vec<(String, String)>,
}

impl MultiRoleJwtConfig {
    /// Create a new empty configuration.
    pub fn new() -> Self {
        Self {
            secrets: Vec::new(),
        }
    }

    /// Add a role with its JWT secret.
    ///
    /// Roles are tried in the order they are added during validation.
    pub fn add_role(mut self, name: &str, secret: String) -> Self {
        self.secrets.push((name.to_string(), secret));
        self
    }

    /// Validate a JWT token against all configured role secrets.
    ///
    /// Returns the first matching `AuthRole`, or `None` if no secret validates the token.
    pub fn validate(&self, token: &str) -> Option<AuthRole> {
        for (role_name, secret) in &self.secrets {
            let decoding_key = DecodingKey::from_secret(secret.as_ref());
            let validation = Validation::default();

            if let Ok(decoded) = decode::<Claims>(token, &decoding_key, &validation) {
                let user_id: i64 = decoded.claims.sub.parse().ok()?;

                return Some(match role_name.as_str() {
                    "user" => AuthRole::User(user_id),
                    "admin" => AuthRole::Admin(user_id),
                    other => AuthRole::Custom(other.to_string(), user_id),
                });
            }
        }
        None
    }
}

impl Default for MultiRoleJwtConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Require admin authentication from the GraphQL context.
///
/// # Arguments
///
/// * `ctx` - The GraphQL context
///
/// # Returns
///
/// The admin's user ID as i64
///
/// # Errors
///
/// Returns UNAUTHORIZED if not authenticated, or FORBIDDEN if not an admin
///
/// # Example
///
/// ```rust,ignore
/// use brylix::auth::roles::require_admin;
///
/// async fn admin_resolver(ctx: &Context<'_>) -> Result<Vec<User>> {
///     let admin_id = require_admin(ctx)?;
///     // Only admins reach here
///     UserService::list_all(db).await
/// }
/// ```
pub fn require_admin(ctx: &Context<'_>) -> Result<i64, GqlError> {
    let data = ctx.data_unchecked::<ContextData>();
    match &data.role {
        Some(role) if role.is_admin() => Ok(role.id()),
        Some(_) => Err(gql_error("FORBIDDEN", "Admin access required")),
        None => Err(gql_unauthorized()),
    }
}

/// Get the authentication role from the GraphQL context.
///
/// Returns `None` if the request is not authenticated or has no role.
///
/// # Example
///
/// ```rust,ignore
/// use brylix::auth::roles::get_auth_role;
///
/// async fn my_resolver(ctx: &Context<'_>) -> Result<String> {
///     if let Some(role) = get_auth_role(ctx) {
///         Ok(format!("You are a {} with id {}", role.role_name(), role.id()))
///     } else {
///         Ok("Not authenticated".to_string())
///     }
/// }
/// ```
pub fn get_auth_role<'a>(ctx: &'a Context<'a>) -> Option<&'a AuthRole> {
    let data = ctx.data_unchecked::<ContextData>();
    data.role.as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_role_user() {
        let role = AuthRole::User(42);
        assert_eq!(role.id(), 42);
        assert!(role.is_user());
        assert!(!role.is_admin());
        assert_eq!(role.role_name(), "user");
    }

    #[test]
    fn test_auth_role_admin() {
        let role = AuthRole::Admin(1);
        assert_eq!(role.id(), 1);
        assert!(role.is_admin());
        assert!(!role.is_user());
        assert_eq!(role.role_name(), "admin");
    }

    #[test]
    fn test_auth_role_custom() {
        let role = AuthRole::Custom("moderator".to_string(), 99);
        assert_eq!(role.id(), 99);
        assert!(!role.is_admin());
        assert!(!role.is_user());
        assert_eq!(role.role_name(), "moderator");
    }

    #[test]
    fn test_multi_role_config_builder() {
        let config = MultiRoleJwtConfig::new()
            .add_role("user", "secret1".to_string())
            .add_role("admin", "secret2".to_string());

        assert_eq!(config.secrets.len(), 2);
    }

    #[test]
    fn test_multi_role_config_default() {
        let config = MultiRoleJwtConfig::default();
        assert!(config.secrets.is_empty());
    }
}
