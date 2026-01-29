//! Task service - business logic layer.

use brylix::prelude::*;
use chrono::NaiveDate;
use sea_orm::DatabaseConnection;

use crate::model::task;
use crate::repository::TaskRepository;

pub struct TaskService;

impl TaskService {
    /// Get task by ID
    pub async fn get_by_id(db: &DatabaseConnection, id: i64) -> DomainResult<task::Model> {
        TaskRepository::find_by_id(db, id)
            .await?
            .ok_or(DomainError::NotFound("Task not found".into()))
    }

    /// List all tasks
    pub async fn list(db: &DatabaseConnection) -> DomainResult<Vec<task::Model>> {
        TaskRepository::find_all(db).await
    }

    /// Get tasks assigned to a user
    pub async fn get_my_tasks(
        db: &DatabaseConnection,
        user_id: i64,
    ) -> DomainResult<Vec<task::Model>> {
        TaskRepository::find_by_assignee(db, user_id).await
    }

    /// Create a new task
    pub async fn create(
        db: &DatabaseConnection,
        title: String,
        description: Option<String>,
        assignee_id: Option<i64>,
        due_date: Option<NaiveDate>,
    ) -> DomainResult<task::Model> {
        // Validate
        if title.trim().is_empty() {
            return Err(DomainError::InvalidInput("Title cannot be empty".into()));
        }

        TaskRepository::create(db, title, description, assignee_id, due_date).await
    }

    /// Update task
    pub async fn update(
        db: &DatabaseConnection,
        id: i64,
        title: Option<String>,
        description: Option<String>,
        status: Option<String>,
        assignee_id: Option<i64>,
        due_date: Option<NaiveDate>,
    ) -> DomainResult<task::Model> {
        // Verify exists
        let _ = Self::get_by_id(db, id).await?;

        // Validate status if provided
        if let Some(ref s) = status {
            let valid = [
                task::status::PENDING,
                task::status::IN_PROGRESS,
                task::status::COMPLETED,
                task::status::CANCELLED,
            ];
            if !valid.contains(&s.as_str()) {
                return Err(DomainError::InvalidInput(format!(
                    "Invalid status. Must be one of: {}",
                    valid.join(", ")
                )));
            }
        }

        TaskRepository::update(db, id, title, description, status, assignee_id, due_date).await
    }

    /// Assign task to a user
    pub async fn assign(
        db: &DatabaseConnection,
        id: i64,
        assignee_id: i64,
    ) -> DomainResult<task::Model> {
        Self::update(db, id, None, None, None, Some(assignee_id), None).await
    }

    /// Mark task as completed
    pub async fn complete(db: &DatabaseConnection, id: i64) -> DomainResult<task::Model> {
        Self::update(
            db,
            id,
            None,
            None,
            Some(task::status::COMPLETED.to_string()),
            None,
            None,
        )
        .await
    }

    /// Delete task
    pub async fn delete(db: &DatabaseConnection, id: i64) -> DomainResult<()> {
        let _ = Self::get_by_id(db, id).await?;
        TaskRepository::delete(db, id).await
    }
}
