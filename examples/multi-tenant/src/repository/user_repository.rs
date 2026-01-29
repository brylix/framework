//! User repository - database access layer.

use brylix::prelude::*;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::model::user::{self, ActiveModel, Column, Entity};

pub struct UserRepository;

impl UserRepository {
    /// Find user by ID
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: i64,
    ) -> DomainResult<Option<user::Model>> {
        Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    /// Find user by email
    pub async fn find_by_email(
        db: &DatabaseConnection,
        email: &str,
    ) -> DomainResult<Option<user::Model>> {
        Entity::find()
            .filter(Column::Email.eq(email))
            .one(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    /// Create a new user
    pub async fn create(
        db: &DatabaseConnection,
        email: String,
        password_hash: String,
        name: String,
    ) -> DomainResult<user::Model> {
        let model = ActiveModel {
            email: Set(email),
            password_hash: Set(password_hash),
            name: Set(name),
            created_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        model
            .insert(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }
}
