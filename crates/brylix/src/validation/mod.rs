//! Input validation functions for Brylix applications.
//!
//! Provides common validation patterns for user input such as email,
//! password, and tenant names.
//!
//! # Usage
//!
//! ```rust
//! use brylix::validation::{validate_email, validate_password, validate_tenant_name};
//!
//! // Validate email
//! validate_email("user@example.com")?;
//!
//! // Validate password (8+ chars, uppercase, lowercase, number, special char)
//! validate_password("SecurePass123!")?;
//!
//! // Validate tenant name (alphanumeric, no SQL injection)
//! validate_tenant_name("my_tenant")?;
//! ```
//!
//! # Regex Patterns
//!
//! This module uses `lazy_static!` for regex compilation with `.unwrap()`.
//! These unwraps are safe because:
//! - The regex patterns are compile-time string literals that have been validated
//! - `lazy_static!` ensures each regex is compiled exactly once at first use
//! - Invalid regex patterns represent programmer errors, not runtime conditions

use lazy_static::lazy_static;
use regex::Regex;

/// Validate an email address format.
///
/// # Arguments
///
/// * `email` - The email address to validate
///
/// # Returns
///
/// `Ok(())` if valid, `Err(String)` with error message if invalid
///
/// # Validation Rules
///
/// - Cannot be empty
/// - Must be less than 255 characters
/// - Must match standard email format
pub fn validate_email(email: &str) -> Result<(), String> {
    if email.is_empty() {
        return Err("Email cannot be empty".to_string());
    }

    if email.len() > 255 {
        return Err("Email must be less than 255 characters".to_string());
    }

    lazy_static! {
        static ref EMAIL_REGEX: Regex = Regex::new(
            r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
        ).unwrap();
    }

    if !EMAIL_REGEX.is_match(email) {
        return Err("Invalid email format".to_string());
    }

    Ok(())
}

/// Validate a password meets security requirements.
///
/// # Arguments
///
/// * `password` - The password to validate
///
/// # Returns
///
/// `Ok(())` if valid, `Err(String)` with error message if invalid
///
/// # Validation Rules
///
/// - Cannot be empty
/// - Must be at least 8 characters
/// - Must be less than 128 characters
/// - Must contain at least one uppercase letter
/// - Must contain at least one lowercase letter
/// - Must contain at least one number
/// - Must contain at least one special character
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.is_empty() {
        return Err("Password cannot be empty".to_string());
    }

    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }

    if password.len() > 128 {
        return Err("Password must be less than 128 characters".to_string());
    }

    // Check for at least one uppercase letter
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Password must contain at least one uppercase letter".to_string());
    }

    // Check for at least one lowercase letter
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err("Password must contain at least one lowercase letter".to_string());
    }

    // Check for at least one digit
    if !password.chars().any(|c| c.is_numeric()) {
        return Err("Password must contain at least one number".to_string());
    }

    // Check for at least one special character
    let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
    if !password.chars().any(|c| special_chars.contains(c)) {
        return Err(
            "Password must contain at least one special character (!@#$%^&*()_+-=[]{}|;:,.<>?)"
                .to_string(),
        );
    }

    Ok(())
}

/// Validate a name (FirstName, LastName, etc.).
///
/// # Arguments
///
/// * `name` - The name to validate
///
/// # Returns
///
/// `Ok(())` if valid, `Err(String)` with error message if invalid
pub fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    if name.len() > 50 {
        return Err("Name must be less than 50 characters".to_string());
    }

    Ok(())
}

/// Validate a tenant name for multi-tenant applications.
///
/// # Arguments
///
/// * `name` - The tenant name to validate
///
/// # Returns
///
/// `Ok(())` if valid, `Err(String)` with error message if invalid
///
/// # Security
///
/// This function prevents SQL injection and path traversal attacks by:
/// - Allowing only alphanumeric characters, underscores, and hyphens
/// - Requiring names to start with alphanumeric character
/// - Blocking path traversal characters (`..`, `/`, `\`)
/// - Limiting length to MySQL's 64 character database name limit
pub fn validate_tenant_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Tenant name cannot be empty".to_string());
    }

    // MySQL database name limit is 64 characters
    if name.len() > 64 {
        return Err("Tenant name must be less than 64 characters".to_string());
    }

    // Prevent path traversal
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err("Tenant name contains invalid characters".to_string());
    }

    // Only allow alphanumeric, underscore, and hyphen
    // Must start with alphanumeric
    lazy_static! {
        static ref TENANT_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9_-]*$").unwrap();
    }

    if !TENANT_REGEX.is_match(name) {
        return Err(
            "Tenant name must start with alphanumeric and contain only letters, numbers, underscores, and hyphens"
                .to_string(),
        );
    }

    Ok(())
}

/// Validate a hostname-safe name (for droplets, servers, etc.).
///
/// # Arguments
///
/// * `name` - The name to validate
///
/// # Returns
///
/// `Ok(())` if valid, `Err(String)` with error message if invalid
pub fn validate_hostname(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    if name.len() > 255 {
        return Err("Name must be less than 255 characters".to_string());
    }

    // Hostnames should be hostname-safe
    lazy_static! {
        static ref NAME_REGEX: Regex =
            Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9]$").unwrap();
    }

    // Single character names are valid
    if name.len() == 1 && name.chars().next().map(|c| c.is_alphanumeric()).unwrap_or(false) {
        return Ok(());
    }

    if !NAME_REGEX.is_match(name) {
        return Err("Name must start and end with alphanumeric characters and contain only letters, numbers, and hyphens".to_string());
    }

    Ok(())
}

/// Validate a string is not empty and within length limits.
///
/// # Arguments
///
/// * `value` - The string to validate
/// * `field_name` - The name of the field (for error messages)
/// * `max_length` - Maximum allowed length
///
/// # Returns
///
/// `Ok(())` if valid, `Err(String)` with error message if invalid
pub fn validate_required_string(value: &str, field_name: &str, max_length: usize) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{} cannot be empty", field_name));
    }

    if value.len() > max_length {
        return Err(format!("{} must be less than {} characters", field_name, max_length));
    }

    Ok(())
}

/// Validate an optional string is within length limits.
///
/// # Arguments
///
/// * `value` - The optional string to validate
/// * `field_name` - The name of the field (for error messages)
/// * `max_length` - Maximum allowed length
///
/// # Returns
///
/// `Ok(())` if valid or None, `Err(String)` with error message if invalid
pub fn validate_optional_string(value: Option<&str>, field_name: &str, max_length: usize) -> Result<(), String> {
    if let Some(v) = value {
        if v.len() > max_length {
            return Err(format!("{} must be less than {} characters", field_name, max_length));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("user.name@example.co.uk").is_ok());
        assert!(validate_email("user+tag@example.com").is_ok());
    }

    #[test]
    fn test_validate_email_invalid() {
        assert!(validate_email("").is_err());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("user@").is_err());
    }

    #[test]
    fn test_validate_password_valid() {
        assert!(validate_password("SecurePass123!").is_ok());
        assert!(validate_password("MyP@ssw0rd").is_ok());
    }

    #[test]
    fn test_validate_password_invalid() {
        assert!(validate_password("").is_err()); // Empty
        assert!(validate_password("short").is_err()); // Too short
        assert!(validate_password("nouppercase123!").is_err()); // No uppercase
        assert!(validate_password("NOLOWERCASE123!").is_err()); // No lowercase
        assert!(validate_password("NoNumbers!").is_err()); // No numbers
        assert!(validate_password("NoSpecial123").is_err()); // No special chars
    }

    #[test]
    fn test_validate_tenant_name_valid() {
        assert!(validate_tenant_name("tenant").is_ok());
        assert!(validate_tenant_name("my_tenant").is_ok());
        assert!(validate_tenant_name("tenant-123").is_ok());
        assert!(validate_tenant_name("Tenant1").is_ok());
    }

    #[test]
    fn test_validate_tenant_name_invalid() {
        assert!(validate_tenant_name("").is_err()); // Empty
        assert!(validate_tenant_name("_tenant").is_err()); // Starts with underscore
        assert!(validate_tenant_name("-tenant").is_err()); // Starts with hyphen
        assert!(validate_tenant_name("tenant/../admin").is_err()); // Path traversal
        assert!(validate_tenant_name("tenant/admin").is_err()); // Contains slash
        assert!(validate_tenant_name("tenant; DROP TABLE").is_err()); // SQL injection attempt
    }

    #[test]
    fn test_validate_hostname_valid() {
        assert!(validate_hostname("a").is_ok());
        assert!(validate_hostname("server-1").is_ok());
        assert!(validate_hostname("web-server-01").is_ok());
    }

    #[test]
    fn test_validate_hostname_invalid() {
        assert!(validate_hostname("").is_err());
        assert!(validate_hostname("-server").is_err());
        assert!(validate_hostname("server-").is_err());
    }
}
