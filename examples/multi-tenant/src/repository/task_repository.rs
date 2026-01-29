//! Task repository - database access layer.

use brylix::prelude::*;
use chrono::{NaiveDate, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::model::task::{self, ActiveModel, Column, Entity};

pub struct TaskRepository;

impl TaskRepository {
    /// Find task by ID
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: i64,
    ) -> DomainResult<Option<task::Model>> {
        Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    /// Find all tasks
    pub async fn find_all(db: &DatabaseConnection) -> DomainResult<Vec<task::Model>> {
        Entity::find()
            .all(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    /// Find tasks by assignee
    pub async fn find_by_assignee(
        db: &DatabaseConnection,
        assignee_id: i64,
    ) -> DomainResult<Vec<task::Model>> {
        Entity::find()
            .filter(Column::AssigneeId.eq(assignee_id))
            .all(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    /// Create a new task
    pub async fn create(
        db: &DatabaseConnection,
        title: String,
        description: Option<String>,
        assignee_id: Option<i64>,
        due_date: Option<NaiveDate>,
    ) -> DomainResult<task::Model> {
        let model = ActiveModel {
            title: Set(title),
            description: Set(description),
            status: Set(task::status::PENDING.to_string()),
            assignee_id: Set(assignee_id),
            due_date: Set(due_date),
            created_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };

        model
            .insert(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
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
        let mut model = ActiveModel {
            id: Set(id),
            updated_at: Set(Some(Utc::now().naive_utc())),
            ..Default::default()
        };

        if let Some(t) = title {
            model.title = Set(t);
        }
        if let Some(d) = description {
            model.description = Set(Some(d));
        }
        if let Some(s) = status {
            model.status = Set(s);
        }
        if let Some(a) = assignee_id {
            model.assignee_id = Set(Some(a));
        }
        if let Some(d) = due_date {
            model.due_date = Set(Some(d));
        }

        model
            .update(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    /// Delete task
    pub async fn delete(db: &DatabaseConnection, id: i64) -> DomainResult<()> {
        Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
