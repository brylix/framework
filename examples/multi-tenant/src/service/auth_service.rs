//! Authentication service

use brylix::prelude::*;
use sea_orm::DatabaseConnection;

use crate::model::user;
use crate::repository::UserRepository;

pub struct AuthService;

impl AuthService {
    /// Login user and return JWT token
    pub async fn login(
        db: &DatabaseConnection,
        _config: &brylix::config::Config,
        tenant: &str,
        email: &str,
        password: &str,
    ) -> DomainResult<String> {
        // Find user by email
        let user = UserRepository::find_by_email(db, email)
            .await?
            .ok_or(DomainError::InvalidCredentials)?;

        // Verify password
        let is_valid = brylix::auth::verify_password(password, &user.password_hash)
            .map_err(|e| DomainError::Internal(e))?;

        if !is_valid {
            return Err(DomainError::InvalidCredentials);
        }

        // Generate JWT token with tenant claim
        let token = brylix::auth::issue_jwt(&user.id.to_string(), Some(tenant))
            .map_err(|e| DomainError::Internal(format!("JWT error: {}", e)))?;

        Ok(token)
    }

    /// Register a new user
    pub async fn register(
        db: &DatabaseConnection,
        email: String,
        password: String,
        name: String,
    ) -> DomainResult<user::Model> {
        // Check if email already exists
        if UserRepository::find_by_email(db, &email).await?.is_some() {
            return Err(DomainError::DuplicateEntry("Email already registered".into()));
        }

        // Validate email
        brylix::validation::validate_email(&email)
            .map_err(|e| DomainError::InvalidInput(e))?;

        // Validate password
        brylix::validation::validate_password(&password)
            .map_err(|e| DomainError::InvalidInput(e))?;

        // Hash password
        let password_hash = brylix::auth::hash_password(&password)
            .map_err(|e| DomainError::Internal(e))?;

        // Create user
        UserRepository::create(db, email, password_hash, name).await
    }

    /// Get current user from context
    pub async fn get_current_user(
        db: &DatabaseConnection,
        user_id: i64,
    ) -> DomainResult<user::Model> {
        UserRepository::find_by_id(db, user_id)
            .await?
            .ok_or(DomainError::NotFound("User not found".into()))
    }
}
