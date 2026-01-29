//! JWT Claims structure and validation.

use crate::config::Config;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

/// JWT Claims structure.
///
/// Contains the standard JWT claims plus optional tenant information
/// for multi-tenant applications.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject - typically the user ID
    pub sub: String,

    /// Tenant database name (for multi-tenant mode)
    #[serde(default)]
    pub tenant: Option<String>,

    /// Expiration timestamp (Unix epoch seconds)
    pub exp: usize,
}

/// Result of JWT validation containing user_id and optional tenant.
#[derive(Debug, Clone)]
pub struct JwtResult {
    /// The user ID extracted from the JWT subject
    pub user_id: Option<String>,

    /// The tenant name from the JWT (multi-tenant mode)
    pub tenant: Option<String>,
}

impl JwtResult {
    /// Create an empty result (no authentication)
    pub fn empty() -> Self {
        Self {
            user_id: None,
            tenant: None,
        }
    }

    /// Check if the result contains a valid user
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }
}

/// Validate a JWT token with the given config.
pub(crate) fn validate_jwt_with_config(token: &str, config: &Config) -> Result<Claims, String> {
    let decoding_key = DecodingKey::from_secret(config.jwt.secret.as_ref());
    let validation = Validation::default();
    let decoded = decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
        tracing::debug!(error = %e, "JWT decode failed");
        "Invalid or expired token"
    })?;
    Ok(decoded.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_result_empty() {
        let result = JwtResult::empty();
        assert!(!result.is_authenticated());
        assert!(result.user_id.is_none());
        assert!(result.tenant.is_none());
    }

    #[test]
    fn test_jwt_result_authenticated() {
        let result = JwtResult {
            user_id: Some("123".to_string()),
            tenant: Some("acme".to_string()),
        };
        assert!(result.is_authenticated());
    }
}
