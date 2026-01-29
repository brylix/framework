//! JWT authentication module for Brylix applications.
//!
//! Provides JWT token issuance, validation, and middleware for Lambda handlers.
//!
//! # Usage
//!
//! ```rust
//! use brylix::auth::{issue_jwt, jwt_middleware, Claims, JwtResult};
//!
//! // Issue a token
//! let token = issue_jwt("user_123", Some("tenant_name"))?;
//!
//! // Validate in middleware (done automatically by the framework)
//! let result = jwt_middleware(&request).await?;
//! if let Some(user_id) = result.user_id {
//!     println!("Authenticated user: {}", user_id);
//! }
//! ```

mod claims;
mod middleware;
mod password;

pub use claims::{Claims, JwtResult};
pub use middleware::jwt_middleware;
pub use password::{hash_password, verify_password, generate_temp_password};

use crate::config::Config;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};

/// Issues a JWT for the given subject and optional tenant.
///
/// # Arguments
///
/// * `sub` - The JWT subject, typically the user ID
/// * `tenant` - Optional tenant name for multi-tenant mode
///
/// # Returns
///
/// The encoded JWT string
///
/// # Errors
///
/// Returns an error if the config is not initialized or token encoding fails
pub fn issue_jwt(sub: &str, tenant: Option<&str>) -> Result<String, String> {
    let config = Config::try_get().ok_or("Config not initialized")?;

    let exp_timestamp = Utc::now() + Duration::days(config.jwt.exp_days);
    let claims = Claims {
        sub: sub.to_string(),
        tenant: tenant.map(String::from),
        exp: exp_timestamp.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt.secret.as_ref()),
    )
    .map_err(|_| "Failed to issue token".to_string())
}

/// Validate a JWT and return the claims.
///
/// # Arguments
///
/// * `token` - The JWT string to validate
///
/// # Returns
///
/// The decoded Claims if valid
///
/// # Errors
///
/// Returns an error if the token is invalid or expired
pub fn validate_jwt(token: &str) -> Result<Claims, String> {
    let config = Config::try_get().ok_or("Config not initialized")?;
    claims::validate_jwt_with_config(token, config)
}
