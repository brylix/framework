//! Admin override support for temporary elevated access.
//!
//! Allows a logged-in user (e.g. cashier) to perform admin-only actions
//! when an admin "taps in" with a short-lived override token.
//!
//! # Usage
//!
//! ```rust,ignore
//! use brylix::auth::admin_override::{
//!     AdminOverrideConfig, issue_admin_override_token, ADMIN_OVERRIDE_HEADER,
//! };
//!
//! // Issue a short-lived override token after verifying admin credentials
//! let config = AdminOverrideConfig::new("admin-secret".to_string());
//! let token = issue_admin_override_token(&config, 42, "John Admin", Some("delete_invoice"))?;
//!
//! // Frontend sends: X-Admin-Override: <token>
//! // Guards like require_admin() will accept this automatically
//! ```

use async_graphql::{Context, Error as GqlError};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use lambda_http::Request;
use serde::{Deserialize, Serialize};

use crate::errors::{gql_error, gql_unauthorized};
use crate::graphql::ContextData;

/// HTTP header name for the admin override token.
pub const ADMIN_OVERRIDE_HEADER: &str = "X-Admin-Override";

/// Default expiry for admin override tokens in seconds.
const DEFAULT_EXPIRY_SECS: i64 = 60;

/// Token type marker to distinguish override tokens from regular admin JWTs.
const ADMIN_OVERRIDE_TOKEN_TYPE: &str = "admin_override";

/// Configuration for admin override tokens.
#[derive(Debug, Clone)]
pub struct AdminOverrideConfig {
    /// The JWT secret used to sign override tokens (same as ADMIN_JWT_SECRET).
    pub secret: String,
    /// How long the override token is valid, in seconds.
    pub expiry_secs: i64,
}

impl AdminOverrideConfig {
    /// Create a new config with default expiry (60 seconds).
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            expiry_secs: DEFAULT_EXPIRY_SECS,
        }
    }

    /// Set the expiry duration in seconds.
    pub fn with_expiry_secs(mut self, secs: i64) -> Self {
        self.expiry_secs = secs;
        self
    }
}

/// JWT claims for an admin override token.
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminOverrideClaims {
    /// Admin user ID (subject).
    pub sub: String,
    /// Admin display name.
    pub admin_name: String,
    /// Optional action being authorized (e.g. "delete_invoice").
    #[serde(default)]
    pub action: Option<String>,
    /// Expiration timestamp (Unix epoch seconds).
    pub exp: usize,
    /// Token type marker — must be "admin_override".
    pub token_type: String,
}

/// Validated admin override information extracted from the token.
#[derive(Debug, Clone)]
pub struct AdminOverride {
    /// The admin's user ID who authorized the override.
    pub admin_id: i64,
    /// The admin's subject string from the token.
    pub admin_sub: String,
    /// The admin's display name.
    pub admin_name: String,
    /// Optional action that was authorized.
    pub action: Option<String>,
}

/// Issue a short-lived admin override token.
///
/// Called after the admin has been authenticated (e.g. password verified).
/// The resulting token is sent by the frontend as the `X-Admin-Override` header.
///
/// # Arguments
///
/// * `config` - Admin override configuration
/// * `admin_id` - The admin's user ID
/// * `admin_name` - The admin's display name (for audit)
/// * `action` - Optional action being authorized
///
/// # Errors
///
/// Returns an error if token encoding fails
pub fn issue_admin_override_token(
    config: &AdminOverrideConfig,
    admin_id: i64,
    admin_name: &str,
    action: Option<&str>,
) -> Result<String, String> {
    let exp = Utc::now().timestamp() + config.expiry_secs;

    let claims = AdminOverrideClaims {
        sub: admin_id.to_string(),
        admin_name: admin_name.to_string(),
        action: action.map(String::from),
        exp: exp as usize,
        token_type: ADMIN_OVERRIDE_TOKEN_TYPE.to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_ref()),
    )
    .map_err(|e| format!("Failed to issue admin override token: {}", e))
}

/// Validate an admin override token and extract the override info.
///
/// # Arguments
///
/// * `token` - The raw JWT string from the `X-Admin-Override` header
/// * `config` - Admin override configuration
///
/// # Errors
///
/// Returns an error if the token is invalid, expired, or not an override token
pub fn validate_admin_override_token(
    token: &str,
    config: &AdminOverrideConfig,
) -> Result<AdminOverride, String> {
    let decoding_key = DecodingKey::from_secret(config.secret.as_ref());
    let validation = Validation::default();

    let decoded = decode::<AdminOverrideClaims>(token, &decoding_key, &validation)
        .map_err(|e| format!("Invalid admin override token: {}", e))?;

    let claims = decoded.claims;

    // Verify this is actually an override token, not a regular admin JWT
    if claims.token_type != ADMIN_OVERRIDE_TOKEN_TYPE {
        return Err("Token is not an admin override token".to_string());
    }

    let admin_id: i64 = claims
        .sub
        .parse()
        .map_err(|_| "Invalid admin ID in override token".to_string())?;

    Ok(AdminOverride {
        admin_id,
        admin_sub: claims.sub,
        admin_name: claims.admin_name,
        action: claims.action,
    })
}

/// Extract the admin override token from the request headers.
///
/// Reads the `X-Admin-Override` header value.
pub fn extract_admin_override_header(request: &Request) -> Option<String> {
    request
        .headers()
        .get(ADMIN_OVERRIDE_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Get the admin override from the GraphQL context, if present.
pub fn get_admin_override<'a>(ctx: &'a Context<'a>) -> Option<&'a AdminOverride> {
    let data = ctx.data_unchecked::<ContextData>();
    data.admin_override.as_ref()
}

/// Require both an authenticated user AND an admin override.
///
/// Returns the user's ID and the admin override info.
/// Useful when you need to know both who performed the action
/// and which admin authorized it.
///
/// # Errors
///
/// Returns UNAUTHORIZED if no user is authenticated,
/// or FORBIDDEN if no admin override is present
pub fn require_auth_with_admin_override(
    ctx: &Context<'_>,
) -> Result<(i64, AdminOverride), GqlError> {
    let data = ctx.data_unchecked::<ContextData>();

    let user_id: i64 = data
        .user
        .as_ref()
        .ok_or_else(gql_unauthorized)?
        .parse()
        .map_err(|_| gql_error("BAD_REQUEST", "Invalid user ID"))?;

    let admin_override = data
        .admin_override
        .as_ref()
        .ok_or_else(|| gql_error("FORBIDDEN", "Admin override required"))?
        .clone();

    Ok((user_id, admin_override))
}

/// Audit trail for admin override actions.
///
/// Captures who performed the action (the logged-in user) and
/// who authorized it (the admin who tapped in).
#[derive(Debug, Clone)]
pub struct AdminOverrideAudit {
    /// The user ID of the person performing the action (e.g. cashier).
    pub actor_user_id: i64,
    /// The admin ID who authorized the action.
    pub authorizer_admin_id: i64,
    /// The admin's display name.
    pub authorizer_name: String,
    /// The action that was authorized.
    pub action: Option<String>,
}

impl AdminOverrideAudit {
    /// Log the audit trail using tracing.
    pub fn log(&self) {
        tracing::info!(
            actor_user_id = self.actor_user_id,
            authorizer_admin_id = self.authorizer_admin_id,
            authorizer_name = %self.authorizer_name,
            action = ?self.action,
            "Admin override action performed"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AdminOverrideConfig {
        AdminOverrideConfig::new("test-admin-secret".to_string())
    }

    #[test]
    fn test_config_defaults() {
        let config = AdminOverrideConfig::new("secret".to_string());
        assert_eq!(config.expiry_secs, 60);
    }

    #[test]
    fn test_config_custom_expiry() {
        let config = AdminOverrideConfig::new("secret".to_string()).with_expiry_secs(120);
        assert_eq!(config.expiry_secs, 120);
    }

    #[test]
    fn test_issue_and_validate_token() {
        let config = test_config();
        let token =
            issue_admin_override_token(&config, 42, "Admin User", Some("delete_invoice")).unwrap();

        let result = validate_admin_override_token(&token, &config).unwrap();
        assert_eq!(result.admin_id, 42);
        assert_eq!(result.admin_sub, "42");
        assert_eq!(result.admin_name, "Admin User");
        assert_eq!(result.action.as_deref(), Some("delete_invoice"));
    }

    #[test]
    fn test_issue_token_without_action() {
        let config = test_config();
        let token = issue_admin_override_token(&config, 1, "Boss", None).unwrap();

        let result = validate_admin_override_token(&token, &config).unwrap();
        assert_eq!(result.admin_id, 1);
        assert_eq!(result.admin_name, "Boss");
        assert!(result.action.is_none());
    }

    #[test]
    fn test_validate_with_wrong_secret() {
        let config = test_config();
        let token =
            issue_admin_override_token(&config, 42, "Admin", None).unwrap();

        let wrong_config = AdminOverrideConfig::new("wrong-secret".to_string());
        let result = validate_admin_override_token(&token, &wrong_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_expired_token() {
        // Use -120 to exceed jsonwebtoken's default leeway (60 seconds)
        let config = AdminOverrideConfig::new("secret".to_string()).with_expiry_secs(-120);
        let token =
            issue_admin_override_token(&config, 42, "Admin", None).unwrap();

        let valid_config = AdminOverrideConfig::new("secret".to_string());
        let result = validate_admin_override_token(&token, &valid_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_regular_jwt_rejected_as_override() {
        // A regular JWT (without token_type: "admin_override") should be rejected
        use jsonwebtoken::{encode, EncodingKey, Header};

        #[derive(serde::Serialize)]
        struct RegularClaims {
            sub: String,
            exp: usize,
        }

        let secret = "test-admin-secret";
        let claims = RegularClaims {
            sub: "42".to_string(),
            exp: (Utc::now().timestamp() + 3600) as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .unwrap();

        let config = AdminOverrideConfig::new(secret.to_string());
        let result = validate_admin_override_token(&token, &config);
        // Should fail because token_type is missing/wrong
        assert!(result.is_err());
    }

    #[test]
    fn test_audit_log() {
        let audit = AdminOverrideAudit {
            actor_user_id: 5,
            authorizer_admin_id: 42,
            authorizer_name: "Admin User".to_string(),
            action: Some("delete_invoice".to_string()),
        };
        // Just verify it doesn't panic
        audit.log();
    }
}
