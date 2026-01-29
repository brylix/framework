//! JWT middleware for extracting authentication from HTTP requests.

use super::claims::{validate_jwt_with_config, JwtResult};
use crate::config::Config;
use lambda_http::Request;

/// JWT middleware that extracts user_id and tenant from Authorization header.
///
/// Returns JwtResult with both values (tenant is None for single-tenant mode).
///
/// # Arguments
///
/// * `request` - The Lambda HTTP request
///
/// # Returns
///
/// A JwtResult containing the user_id and tenant if authenticated
///
/// # Errors
///
/// Returns an error if the token is invalid
pub async fn jwt_middleware(request: &Request) -> Result<JwtResult, String> {
    let config = Config::try_get().ok_or("Config not initialized")?;

    if let Some(auth_header) = request.headers().get("Authorization") {
        if let Ok(auth_header_str) = auth_header.to_str() {
            if let Some(token) = auth_header_str.strip_prefix("Bearer ") {
                match validate_jwt_with_config(token, config) {
                    Ok(claims) => {
                        return Ok(JwtResult {
                            user_id: Some(claims.sub),
                            tenant: claims.tenant,
                        });
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }
    }

    Ok(JwtResult::empty())
}

/// Extract Bearer token from Authorization header.
///
/// # Arguments
///
/// * `request` - The Lambda HTTP request
///
/// # Returns
///
/// The token string if present
pub fn extract_bearer_token(request: &Request) -> Option<String> {
    request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}
