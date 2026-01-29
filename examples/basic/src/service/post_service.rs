//! Post service - business logic layer.

use brylix::prelude::*;
use sea_orm::DatabaseConnection;

use crate::model::post;
use crate::repository::PostRepository;

pub struct PostService;

impl PostService {
    /// Get post by ID
    pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> DomainResult<post::Model> {
        PostRepository::find_by_id(db, id)
            .await?
            .ok_or(DomainError::NotFound("Post not found".into()))
    }

    /// List all posts
    pub async fn list(db: &DatabaseConnection) -> DomainResult<Vec<post::Model>> {
        PostRepository::find_all(db).await
    }

    /// Create a new post
    pub async fn create(
        db: &DatabaseConnection,
        title: String,
        content: String,
    ) -> DomainResult<post::Model> {
        // Validate input
        if title.trim().is_empty() {
            return Err(DomainError::InvalidInput("Title cannot be empty".into()));
        }
        if content.trim().is_empty() {
            return Err(DomainError::InvalidInput("Content cannot be empty".into()));
        }

        PostRepository::create(db, title, content).await
    }

    /// Update post
    pub async fn update(
        db: &DatabaseConnection,
        id: i64,
        title: Option<String>,
        content: Option<String>,
        published: Option<bool>,
    ) -> DomainResult<post::Model> {
        // Verify exists
        let _ = Self::get_by_id(db, id).await?;

        // Validate
        if let Some(ref t) = title {
            if t.trim().is_empty() {
                return Err(DomainError::InvalidInput("Title cannot be empty".into()));
            }
        }

        PostRepository::update(db, id, title, content, published).await
    }

    /// Publish a post
    pub async fn publish(db: &DatabaseConnection, id: i64) -> DomainResult<post::Model> {
        Self::update(db, id, None, None, Some(true)).await
    }

    /// Delete post
    pub async fn delete(db: &DatabaseConnection, id: i64) -> DomainResult<()> {
        // Verify exists
        let _ = Self::get_by_id(db, id).await?;
        PostRepository::delete(db, id).await
    }
}
