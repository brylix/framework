//! Post repository - database access layer.

use brylix::prelude::*;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

use crate::model::post::{self, ActiveModel, Entity};

pub struct PostRepository;

impl PostRepository {
    /// Find post by ID
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: i64,
    ) -> DomainResult<Option<post::Model>> {
        Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    /// Find all posts
    pub async fn find_all(db: &DatabaseConnection) -> DomainResult<Vec<post::Model>> {
        Entity::find()
            .all(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    /// Create a new post
    pub async fn create(
        db: &DatabaseConnection,
        title: String,
        content: String,
    ) -> DomainResult<post::Model> {
        let model = ActiveModel {
            title: Set(title),
            content: Set(content),
            published: Set(false),
            created_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        model
            .insert(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    /// Update post
    pub async fn update(
        db: &DatabaseConnection,
        id: i64,
        title: Option<String>,
        content: Option<String>,
        published: Option<bool>,
    ) -> DomainResult<post::Model> {
        let mut model = ActiveModel {
            id: Set(id),
            updated_at: Set(Some(Utc::now().naive_utc())),
            ..Default::default()
        };

        if let Some(t) = title {
            model.title = Set(t);
        }
        if let Some(c) = content {
            model.content = Set(c);
        }
        if let Some(p) = published {
            model.published = Set(p);
        }

        model
            .update(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    /// Delete post
    pub async fn delete(db: &DatabaseConnection, id: i64) -> DomainResult<()> {
        Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
